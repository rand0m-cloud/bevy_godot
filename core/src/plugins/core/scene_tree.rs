use crate::prelude::{
    bevy_prelude::{debug, CoreStage, EventReader, EventWriter, NonSendMut},
    godot_prelude::{FromVariant, SubClass, ToVariant, Variant, VariantArray},
    *,
};
use bevy::ecs::event::Events;
use bevy::ecs::system::SystemParam;
use gdnative::api::Engine;
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct GodotSceneTreePlugin;

impl Plugin for GodotSceneTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(initialize_scene_tree)
            .add_startup_system(connect_scene_tree)
            .add_system_to_stage(
                CoreStage::First,
                write_scene_tree_events.before(Events::<SceneTreeEvent>::update_system),
            )
            .add_system_to_stage(
                CoreStage::First,
                read_scene_tree_events.after(Events::<SceneTreeEvent>::update_system),
            )
            .add_event::<SceneTreeEvent>()
            .init_non_send_resource::<SceneTreeRefImpl>();
    }
}

#[derive(SystemParam)]
pub struct SceneTreeRef<'w, 's> {
    godot_ref: NonSendMut<'w, SceneTreeRefImpl>,
    #[system_param(ignore)]
    phantom: PhantomData<&'s ()>,
}

impl<'w, 's> SceneTreeRef<'w, 's> {
    pub fn get(&mut self) -> TRef<SceneTree> {
        self.godot_ref.0.get()
    }
}

#[doc(hidden)]
pub struct SceneTreeRefImpl(ErasedGodotRef);

impl SceneTreeRefImpl {
    fn get_ref() -> Ref<SceneTree> {
        unsafe {
            let engine = Engine::godot_singleton();
            engine
                .get_main_loop()
                .and_then(|lp| Some(lp.assume_unique().cast::<SceneTree>()?.into_shared()))
                .unwrap()
        }
    }
}

impl Default for SceneTreeRefImpl {
    fn default() -> Self {
        Self(unsafe { ErasedGodotRef::new(Self::get_ref().assume_unique()) })
    }
}

fn initialize_scene_tree(mut scene_tree: SceneTreeRef, mut events: ResMut<Events<SceneTreeEvent>>) {
    fn traverse(node: TRef<Node>, events: &mut ResMut<Events<SceneTreeEvent>>) {
        unsafe {
            events.send(SceneTreeEvent {
                node: ErasedGodotRef::from_instance_id(node.get_instance_id()),
                event_type: SceneTreeEventType::NodeAdded,
            });

            node.get_children()
                .assume_unique()
                .into_iter()
                .for_each(|child| {
                    let child = child.to_object::<Node>().unwrap().assume_safe();
                    traverse(child, events);
                });
        }
    }

    unsafe {
        let root = scene_tree.get().root().unwrap().assume_safe();
        traverse(root.upcast(), &mut events);
    }
}

#[derive(Debug)]
pub struct SceneTreeEvent {
    pub node: ErasedGodotRef,
    pub event_type: SceneTreeEventType,
}

#[derive(ToVariant, FromVariant, Copy, Clone, Debug)]
pub enum SceneTreeEventType {
    NodeAdded,
    NodeRemoved,
    NodeRenamed,
}

fn connect_scene_tree(mut scene_tree: SceneTreeRef) {
    let scene_tree = scene_tree.get();
    let watcher = unsafe {
        scene_tree
            .root()
            .unwrap()
            .assume_safe()
            .get_node("Autoload/SceneTreeWatcher")
            .unwrap()
    };

    scene_tree
        .connect(
            "node_added",
            watcher,
            "scene_tree_event",
            VariantArray::from_iter(&[SceneTreeEventType::NodeAdded]).into_shared(),
            0,
        )
        .unwrap();

    scene_tree
        .connect(
            "node_removed",
            watcher,
            "scene_tree_event",
            VariantArray::from_iter(&[SceneTreeEventType::NodeRemoved]).into_shared(),
            0,
        )
        .unwrap();

    scene_tree
        .connect(
            "node_renamed",
            watcher,
            "scene_tree_event",
            VariantArray::from_iter(&[SceneTreeEventType::NodeRenamed]).into_shared(),
            0,
        )
        .unwrap();
}

#[derive(Component, Debug)]
pub struct Groups {
    groups: Vec<String>,
}

impl<T: SubClass<Node>> From<&T> for Groups {
    fn from(node: &T) -> Self {
        Groups {
            groups: node
                .upcast::<Node>()
                .get_groups()
                .iter()
                .map(|variant| variant.try_to::<String>().unwrap())
                .collect(),
        }
    }
}

impl std::ops::Deref for Groups {
    type Target = [String];
    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}

impl Groups {
    pub fn is(&self, group_name: &str) -> bool {
        self.groups.iter().any(|name| name == group_name)
    }
}

#[doc(hidden)]
pub struct SceneTreeEventReader(pub std::sync::mpsc::Receiver<SceneTreeEvent>);

fn write_scene_tree_events(
    event_reader: NonSendMut<SceneTreeEventReader>,
    mut event_writer: EventWriter<SceneTreeEvent>,
) {
    event_writer.send_batch(event_reader.0.try_iter());
}

fn read_scene_tree_events(
    mut commands: Commands,
    mut scene_tree: SceneTreeRef,
    mut event_reader: EventReader<SceneTreeEvent>,
    entities: Query<(&mut ErasedGodotRef, Entity)>,
) {
    let mut ent_mapping = entities
        .iter()
        .map(|(reference, ent)| (reference.instance_id(), ent))
        .collect::<HashMap<_, _>>();

    for event in event_reader.iter() {
        let mut node = event.node.clone();

        let ent = ent_mapping.get(&node.instance_id()).cloned();
        let scene_root = unsafe { scene_tree.get().root().unwrap().assume_safe() };
        let collision_watcher = unsafe {
            scene_root
                .get_node("/root/Autoload/CollisionWatcher")
                .unwrap()
                .assume_safe()
        };

        match event.event_type {
            SceneTreeEventType::NodeAdded => {
                let mut ent = if let Some(ent) = ent {
                    commands.entity(ent)
                } else {
                    commands.spawn()
                };

                ent.insert(ErasedGodotRef::clone(&node))
                    .insert(Name::from(node.get::<Node>().name().to_string()))
                    .insert(Children::default());

                if node.instance_id() != scene_root.get_instance_id() {
                    let parent = unsafe {
                        node.get::<Node>()
                            .get_parent()
                            .unwrap()
                            .assume_safe()
                            .get_instance_id()
                    };
                    ent.insert(Parent(*ent_mapping.get(&parent).unwrap()));
                }

                if let Some(spatial) = node.try_get::<Spatial>() {
                    ent.insert(Transform::from(spatial.transform().to_bevy_transform()));
                }

                if let Some(physics_body) = node.try_get::<PhysicsBody>() {
                    if physics_body.has_signal("body_entered") {
                        debug!(target: "godot_scene_tree_collisions", body_id = physics_body.get_instance_id(), "has body_entered signal");
                        physics_body
                            .connect(
                                "body_entered",
                                collision_watcher,
                                "collision_event",
                                VariantArray::from_iter(&[
                                    Variant::new(physics_body.claim()),
                                    Variant::new(CollisionEventType::Started),
                                ])
                                .into_shared(),
                                0,
                            )
                            .unwrap();
                        physics_body
                            .connect(
                                "body_exited",
                                collision_watcher,
                                "collision_event",
                                VariantArray::from_iter(&[
                                    Variant::new(physics_body.claim()),
                                    Variant::new(CollisionEventType::Ended),
                                ])
                                .into_shared(),
                                0,
                            )
                            .unwrap();

                        ent.insert(Collisions::default());
                    }
                }

                ent.insert(Groups::from(&*node.get::<Node>()));

                let ent = ent.id();
                ent_mapping.insert(node.instance_id(), ent);
            }
            SceneTreeEventType::NodeRemoved => {
                commands.entity(ent.unwrap()).despawn_recursive();
            }
            SceneTreeEventType::NodeRenamed => {
                commands
                    .entity(ent.unwrap())
                    .insert(Name::from(node.get::<Node>().name().to_string()));
            }
        }
    }
}
