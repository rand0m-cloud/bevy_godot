use crate::prelude::*;
use bevy::ecs::event::Events;
use gdnative::api::Engine;
use std::collections::{HashMap, HashSet};

pub struct GodotSceneTreePlugin;

impl Plugin for GodotSceneTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::First, on_scene_tree_change)
            .add_startup_system(add_scene_root)
            .add_system(scene_tree_changed)
            .add_system(add_godot_names)
            .init_resource::<Events<SceneTreeChanged>>()
            .init_resource::<SceneTreeRef>();
    }
}

pub struct SceneTreeRef(pub OwnedGodotRef<SceneTree>);

impl SceneTreeRef {
    pub fn get_ref() -> Ref<SceneTree> {
        unsafe {
            let engine = Engine::godot_singleton();
            engine
                .get_main_loop()
                .and_then(|lp| Some(lp.assume_unique().cast::<SceneTree>()?.into_shared()))
                .unwrap()
        }
    }

    pub fn get(&self) -> TRef<'_, SceneTree> {
        self.0.get()
    }
}

impl Default for SceneTreeRef {
    fn default() -> Self {
        Self(OwnedGodotRef::from_ref(Self::get_ref()))
    }
}

pub struct SceneTreeChanged;

fn scene_tree_changed(
    mut writer: EventWriter<SceneTreeChanged>,
    channel: NonSend<std::sync::mpsc::Receiver<()>>,
) {
    if channel.try_recv().is_ok() {
        writer.send(SceneTreeChanged);
    }
}

fn add_scene_root(mut commands: Commands, scene_tree: Res<SceneTreeRef>, _godot_lock: GodotLock) {
    let root = scene_tree.get().root().unwrap();
    commands
        .spawn()
        .insert(ErasedGodotRef::new(unsafe { root.assume_unique() }))
        .insert(Name::from("/root"))
        .insert(Children::default());
}

fn on_scene_tree_change(
    mut commands: Commands,
    mut reader: EventReader<SceneTreeChanged>,
    scene_tree: Res<SceneTreeRef>,
    entities: Query<(&ErasedGodotRef, Entity, &Children)>,
    _godot_lock: GodotLock,
) {
    if reader.is_empty() {
        return;
    }
    for _e in reader.iter() {}

    unsafe fn traverse(node: TRef<Node>, instances: &mut HashMap<i64, i64>) {
        let parent_id = node.get_instance_id();
        node.get_children()
            .assume_unique()
            .into_iter()
            .for_each(|child| {
                let child = child.to_object::<Node>().unwrap().assume_safe();
                let child_id = child.get_instance_id();
                instances.insert(child_id, parent_id);

                traverse(child, instances);
            });
    }

    let mut instances = HashMap::new();
    unsafe {
        let root = scene_tree.get().root().unwrap().assume_safe();
        traverse(root.upcast(), &mut instances);
    }

    let mut instance_id_mapping = entities
        .iter()
        .map(|(reference, ent, _)| (reference.instance_id(), ent))
        .collect::<HashMap<_, _>>();

    let registered_children = entities
        .iter()
        .flat_map(|(reference, _, children)| {
            children.iter().map(|child_ent| {
                let (child_reference, _, _) = entities.get(*child_ent).unwrap();
                (child_reference.instance_id(), reference.instance_id())
            })
        })
        .collect::<HashMap<_, _>>();

    let mut moved_entities = Vec::new();
    let mut new_entities = Vec::new();
    for (child, parent) in &instances {
        if registered_children.get(child) == Some(parent) {
            continue;
        } else if let Some(old_parent) = registered_children.get(child) {
            moved_entities.push((child, (old_parent, parent)));
        } else {
            new_entities.push((child, parent));
        }
    }

    let deleted_entities: Vec<i64> = {
        let registered = registered_children.keys().copied().collect::<HashSet<_>>();
        let new = instances.keys().copied().collect();
        registered.difference(&new).copied().collect()
    };

    for instance_id in new_entities
        .iter()
        .flat_map(|(child, parent)| vec![child, parent])
    {
        if instance_id_mapping.get(instance_id).is_none() {
            let node = ErasedGodotRef::from_instance_id(**instance_id);
            let name = node.get::<Node>().name().to_string();
            let ent = commands
                .spawn()
                .insert(node)
                .insert(Children::default())
                .insert(Name::new(name))
                .id();
            instance_id_mapping.insert(**instance_id, ent);
        }
    }

    for (child, parent) in new_entities {
        let ent = *instance_id_mapping.get(child).unwrap();
        let ent_parent = *instance_id_mapping.get(parent).unwrap();
        commands.entity(ent_parent).push_children(&[ent]);
    }

    for ent in deleted_entities {
        let entity = *instance_id_mapping.get(&ent).unwrap();
        commands.entity(entity).despawn();
    }

    for (child_id, (old_parent_id, parent_id)) in moved_entities {
        let [entity, old_parent_ent, parent_ent]: [Entity; 3] =
            [child_id, old_parent_id, parent_id]
                .iter()
                .map(|id| *instance_id_mapping.get(id).unwrap())
                .collect::<Vec<Entity>>()
                .try_into()
                .unwrap();

        commands.entity(old_parent_ent).remove_children(&[entity]);
        commands.entity(parent_ent).push_children(&[entity]);
    }
}

fn add_godot_names(
    mut commands: Commands,
    missing: Query<(Entity, &ErasedGodotRef), (Without<Name>, With<Children>)>,
) {
    for (ent, reference) in missing.iter() {
        commands
            .entity(ent)
            .insert(Name::from(reference.get::<Node>().name().to_string()));
    }
}
