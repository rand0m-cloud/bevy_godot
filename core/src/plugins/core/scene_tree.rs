use crate::prelude::*;
use bevy::ecs::system::SystemParam;
use gdnative::api::Engine;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

pub struct GodotSceneTreePlugin;

impl Plugin for GodotSceneTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(add_scene_root)
            .add_system_to_stage(GodotStage::SceneTreeUpdate, on_scene_tree_change)
            .add_system_to_stage(GodotStage::BeforeBevy, add_godot_names)
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

fn add_scene_root(mut commands: Commands, mut scene_tree: SceneTreeRef) {
    let root = scene_tree.get().root().unwrap();
    commands
        .spawn()
        .insert(unsafe { ErasedGodotRef::new(root.assume_unique()) })
        .insert(Name::from("/root"))
        .insert(Children::default());
}

fn on_scene_tree_change(
    mut commands: Commands,
    mut scene_tree: SceneTreeRef,
    entities: Query<(&mut ErasedGodotRef, Entity, &Children)>,
) {
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
            let mut node = unsafe { ErasedGodotRef::from_instance_id(**instance_id) };
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
    mut missing: Query<(Entity, &mut ErasedGodotRef), (Without<Name>, With<Children>)>,
    _scene_tree: SceneTreeRef,
) {
    for (ent, mut reference) in missing.iter_mut() {
        commands
            .entity(ent)
            .insert(Name::from(reference.get::<Node>().name().to_string()));
    }
}
