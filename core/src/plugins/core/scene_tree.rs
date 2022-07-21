use crate::prelude::{
    bevy_prelude::{EventReader, EventWriter, NonSendMut},
    godot_prelude::*,
    *,
};
use bevy::ecs::system::SystemParam;
use gdnative::api::Engine;
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct GodotSceneTreePlugin;

impl Plugin for GodotSceneTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(initialize_scene_tree)
            .add_startup_system(connect_scene_tree)
            .add_system_to_stage(GodotStage::SceneTreeUpdate, write_scene_tree_events)
            .add_system_to_stage(
                GodotStage::SceneTreeUpdate,
                read_scene_tree_events.after(write_scene_tree_events),
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

fn initialize_scene_tree(mut commands: Commands, mut scene_tree: SceneTreeRef) {
    unsafe fn traverse(
        node: TRef<Node>,
        ent_mapping: &mut HashMap<i64, Entity>,
        commands: &mut Commands,
    ) {
        let mut ent = commands.spawn();
        ent.insert(ErasedGodotRef::new(node.assume_unique()))
            .insert(Name::from(node.name().to_string()))
            .insert(Children::default());

        if let Some(parent_ent) = node
            .get_parent()
            .and_then(|parent| ent_mapping.get(&parent.assume_safe().get_instance_id()))
        {
            ent.insert(Parent(*parent_ent));
        }

        let ent = ent.id();
        ent_mapping.insert(node.get_instance_id(), ent);

        node.get_children()
            .assume_unique()
            .into_iter()
            .for_each(|child| {
                let child = child.to_object::<Node>().unwrap().assume_safe();
                traverse(child, ent_mapping, commands);
            });
    }

    unsafe {
        let root = scene_tree.get().root().unwrap().assume_safe();
        let mut ent_mapping: HashMap<i64, Entity> = HashMap::new();
        traverse(root.upcast(), &mut ent_mapping, &mut commands);
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
    mut event_reader: EventReader<SceneTreeEvent>,
    entities: Query<(&mut ErasedGodotRef, Entity)>,
) {
    for event in event_reader.iter() {
        let mut node = event.node.clone();

        let mut ent_mapping = entities
            .iter()
            .map(|(reference, ent)| (reference.instance_id(), ent))
            .collect::<HashMap<_, _>>();
        let ent = ent_mapping.get(&node.instance_id()).cloned();

        match event.event_type {
            SceneTreeEventType::NodeAdded => {
                let mut ent = if let Some(ent) = ent {
                    commands.entity(ent)
                } else {
                    commands.spawn()
                };

                ent.insert(node.clone())
                    .insert(Name::from(node.get::<Node>().name().to_string()))
                    .insert(Children::default());

                let parent = unsafe {
                    node.get::<Node>()
                        .get_parent()
                        .unwrap()
                        .assume_safe()
                        .get_instance_id()
                };
                ent.insert(Parent(*ent_mapping.get(&parent).unwrap()));

                if let Some(spatial) = node.try_get::<Spatial>() {
                    ent.insert(spatial.transform().to_bevy_transform());
                }

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
