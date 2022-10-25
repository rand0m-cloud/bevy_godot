use crate::prelude::{
    godot_prelude::{Engine, FromVariant, SubClass, ToVariant, Variant, VariantArray, Viewport},
    *,
};
use bevy::ecs::system::SystemParam;
use std::{collections::HashMap, marker::PhantomData};

pub struct GodotSceneTreePlugin;

impl Plugin for GodotSceneTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, initialize_scene_tree)
            .add_startup_system_to_stage(StartupStage::PreStartup, connect_scene_tree)
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

    pub fn get_current_scene(&mut self) -> TRef<Node> {
        unsafe { self.get().current_scene().unwrap().assume_safe() }
    }

    pub fn get_root(&mut self) -> TRef<Viewport> {
        unsafe { self.get().root().unwrap().assume_safe() }
    }

    pub fn add_to_scene<T: SubClass<Node>>(&mut self, node: TRef<T>) {
        self.get_current_scene().add_child(node.upcast(), true);
    }

    pub fn add_to_root<T: SubClass<Node>>(&mut self, node: TRef<T>) {
        self.get_root().add_child(node.upcast(), true);
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

fn initialize_scene_tree(
    mut commands: Commands,
    mut scene_tree: SceneTreeRef,
    mut entities: Query<(&mut ErasedGodotRef, Entity)>,
) {
    fn traverse(node: TRef<Node>, events: &mut Vec<SceneTreeEvent>) {
        unsafe {
            events.push(SceneTreeEvent {
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
        let mut events = vec![];
        traverse(root.upcast(), &mut events);

        create_scene_tree_entity(&mut commands, events, &mut scene_tree, &mut entities);
    }
}

#[derive(Debug, Clone)]
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
            VariantArray::from_iter([SceneTreeEventType::NodeAdded]).into_shared(),
            0,
        )
        .unwrap();

    scene_tree
        .connect(
            "node_removed",
            watcher,
            "scene_tree_event",
            VariantArray::from_iter([SceneTreeEventType::NodeRemoved]).into_shared(),
            0,
        )
        .unwrap();

    scene_tree
        .connect(
            "node_renamed",
            watcher,
            "scene_tree_event",
            VariantArray::from_iter([SceneTreeEventType::NodeRenamed]).into_shared(),
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

fn create_scene_tree_entity(
    commands: &mut Commands,
    events: impl IntoIterator<Item = SceneTreeEvent>,
    scene_tree: &mut SceneTreeRef,
    entities: &mut Query<(&mut ErasedGodotRef, Entity)>,
) {
    let mut ent_mapping = entities
        .iter()
        .map(|(reference, ent)| (reference.instance_id(), ent))
        .collect::<HashMap<_, _>>();
    let scene_root = unsafe { scene_tree.get().root().unwrap().assume_safe() };
    let collision_watcher = unsafe {
        scene_root
            .get_node("/root/Autoload/CollisionWatcher")
            .unwrap()
            .assume_safe()
    };

    for event in events.into_iter() {
        trace!(target: "godot_scene_tree_events", event = ?event);

        let mut node = event.node.clone();
        let ent = ent_mapping.get(&node.instance_id()).cloned();

        match event.event_type {
            SceneTreeEventType::NodeAdded => {
                let mut ent = if let Some(ent) = ent {
                    commands.entity(ent)
                } else {
                    commands.spawn()
                };

                ent.insert(ErasedGodotRef::clone(&node))
                    .insert(Name::from(node.get::<Node>().name().to_string()));

                if let Some(spatial) = node.try_get::<Spatial>() {
                    ent.insert(Transform::from(spatial.transform().to_bevy_transform()));
                }

                if let Some(node2d) = node.try_get::<Node2D>() {
                    // gdnative's Transform2D has buggy modifiers
                    let mut transform = GodotTransform2D::IDENTITY.translated(node2d.position());
                    transform.set_scale(node2d.scale());
                    transform.set_rotation(node2d.rotation() as f32);

                    ent.insert(Transform2D::from(transform));
                }

                let node = node.get::<Node>();

                if node.has_signal("body_entered") {
                    debug!(target: "godot_scene_tree_collisions", body_id = node.get_instance_id(), "has body_entered signal");
                    node.connect(
                        "body_entered",
                        collision_watcher,
                        "collision_event",
                        VariantArray::from_iter(&[
                            Variant::new(node.claim()),
                            Variant::new(CollisionEventType::Started),
                        ])
                        .into_shared(),
                        0,
                    )
                    .unwrap();
                    node.connect(
                        "body_exited",
                        collision_watcher,
                        "collision_event",
                        VariantArray::from_iter(&[
                            Variant::new(node.claim()),
                            Variant::new(CollisionEventType::Ended),
                        ])
                        .into_shared(),
                        0,
                    )
                    .unwrap();

                    ent.insert(Collisions::default());
                }

                ent.insert(Groups::from(&*node));

                let ent = ent.id();
                ent_mapping.insert(node.get_instance_id(), ent);

                if node.get_instance_id() != scene_root.get_instance_id() {
                    let parent =
                        unsafe { node.get_parent().unwrap().assume_safe().get_instance_id() };
                    commands
                        .entity(*ent_mapping.get(&parent).unwrap())
                        .push_children(&[ent]);
                }
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

fn read_scene_tree_events(
    mut commands: Commands,
    mut scene_tree: SceneTreeRef,
    mut event_reader: EventReader<SceneTreeEvent>,
    mut entities: Query<(&mut ErasedGodotRef, Entity)>,
) {
    create_scene_tree_entity(
        &mut commands,
        event_reader.iter().cloned(),
        &mut scene_tree,
        &mut entities,
    );
}
