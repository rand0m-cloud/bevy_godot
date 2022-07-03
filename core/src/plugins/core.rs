use bevy::ecs::event::Events;
use bevy::{app::*, prelude::*};
use gdnative::prelude::Node;
use gdnative::{
    api::{Engine, Object, Reference, SceneTree, Spatial},
    core_types::{Basis, Vector3},
    object::{
        bounds::RefImplBound,
        bounds::SafeAsRaw,
        memory::{ManuallyManaged, RefCounted},
        ownership::Ownership,
        GodotObject, Ref, SubClass, TRef,
    },
};
use std::collections::HashMap;
use std::collections::HashSet;
use std::marker::PhantomData;

pub struct GodotCorePlugin;

impl Plugin for GodotCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy::core::CorePlugin)
            .add_system_to_stage(CoreStage::PostUpdate, post_update_godot_transforms)
            .add_system_to_stage(CoreStage::PreUpdate, pre_update_godot_transforms)
            .add_system_to_stage(CoreStage::PostUpdate, set_godot_transforms)
            .add_system_to_stage(CoreStage::First, on_scene_tree_change)
            .insert_non_send_resource(GodotLockImpl)
            .add_startup_system(add_scene_root)
            .add_system(scene_tree_changed)
            .add_system(add_godot_names)
            .init_resource::<Events<SceneTreeChanged>>()
            .init_resource::<SceneTreeRef>();
    }
}

#[doc(hidden)]
pub struct GodotLockImpl;

pub type GodotLock<'a> = NonSendMut<'a, GodotLockImpl>;

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct OwnedGodotRef<T: GodotObject + 'static> {
    #[reflect(ignore)]
    object: Option<Ref<Object>>,
    #[reflect(ignore)]
    phantom: PhantomData<Ref<T>>,
}

impl<T: GodotObject + 'static> Default for OwnedGodotRef<T> {
    fn default() -> Self {
        Self {
            object: None,
            phantom: PhantomData,
        }
    }
}

impl<T: GodotObject<Memory = ManuallyManaged> + SubClass<Object>> OwnedGodotRef<T> {
    pub fn get(&self) -> TRef<'_, T> {
        unsafe {
            self.object
                .as_ref()
                .unwrap()
                .assume_safe()
                .cast()
                .unwrap_unchecked()
        }
    }

    pub fn from_ref(reference: Ref<T>) -> Self {
        unsafe {
            Self {
                object: Some(reference.assume_unique().upcast::<Object>().into_shared()),
                ..Self::default()
            }
        }
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct GodotRef<T: GodotObject + 'static> {
    #[reflect(ignore)]
    object: Option<Ref<Reference>>,
    #[reflect(ignore)]
    phantom: PhantomData<Ref<T>>,
}

impl<T: GodotObject + 'static> Default for GodotRef<T> {
    fn default() -> Self {
        Self {
            object: None,
            phantom: PhantomData,
        }
    }
}

impl<T: GodotObject<Memory = RefCounted> + SubClass<Reference>> GodotRef<T> {
    pub fn get(&self) -> TRef<'_, T> {
        unsafe {
            self.object
                .as_ref()
                .unwrap()
                .assume_safe()
                .cast()
                .unwrap_unchecked()
        }
    }

    pub fn from_ref(reference: Ref<T>) -> Self {
        unsafe {
            Self {
                object: Some(
                    reference
                        .assume_unique()
                        .upcast::<Reference>()
                        .into_shared(),
                ),
                ..Self::default()
            }
        }
    }
}

#[derive(Component, Reflect, Clone, Default, Debug)]
#[reflect(Component)]
pub struct ErasedGodotRef {
    #[reflect(ignore)]
    object_id: i64,
    class_name: String,
}

impl ErasedGodotRef {
    pub fn get<T: GodotObject>(&self) -> TRef<'_, T> {
        self.try_get()
            .unwrap_or_else(|| panic!("failed to get godot ref as {}", std::any::type_name::<T>()))
    }

    pub fn try_get<T: GodotObject>(&self) -> Option<TRef<'_, T>> {
        unsafe { TRef::try_from_instance_id(self.object_id) }
    }

    pub fn new<T: GodotObject + SubClass<Object>, Own: Ownership>(reference: Ref<T, Own>) -> Self
    where
        RefImplBound: SafeAsRaw<<T as GodotObject>::Memory, Own>,
    {
        let obj = Object::cast_ref(reference.as_raw().cast().unwrap());
        Self::from_instance_id(obj.get_instance_id())
    }

    pub fn instance_id(&self) -> i64 {
        self.object_id
    }

    pub fn from_instance_id(id: i64) -> Self {
        let obj: TRef<Object> = unsafe { TRef::from_instance_id(id) };
        let object_id = obj.get_instance_id();
        let class_name = obj.get_class().to_string();
        Self {
            object_id,
            class_name,
        }
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

pub trait IntoBevyTransform {
    fn to_bevy_transform(self) -> bevy::prelude::Transform;
}

impl IntoBevyTransform for gdnative::prelude::Transform {
    fn to_bevy_transform(self) -> bevy::prelude::Transform {
        let quat = self.basis.to_quat();
        let quat = Quat::from_xyzw(quat.x, quat.y, quat.y, quat.w);

        let scale = self.basis.scale();
        let scale = Vec3::new(scale.x, scale.y, scale.z);

        let origin = Vec3::new(self.origin.x, self.origin.y, self.origin.z);

        Transform {
            rotation: quat,
            translation: origin,
            scale,
        }
    }
}

pub trait IntoGodotTransform {
    fn to_godot_transform(self) -> gdnative::prelude::Transform;
}

impl IntoGodotTransform for bevy::prelude::Transform {
    fn to_godot_transform(self) -> gdnative::prelude::Transform {
        use gdnative::prelude::Quat;

        let [x, y, z, w] = self.rotation.to_array();
        let quat = Quat::new(x, y, z, w);

        let [x, y, z] = self.scale.to_array();
        let scale = Vector3::new(x, y, z);

        let basis = Basis::from_quat(quat).scaled(scale);

        let [x, y, z] = self.translation.to_array();
        gdnative::prelude::Transform {
            basis,
            origin: Vector3::new(x, y, z),
        }
    }
}

fn post_update_godot_transforms(
    entities: Query<(&Transform, &ErasedGodotRef), Changed<Transform>>,
    _godot_lock: GodotLock,
) {
    for (transform, reference) in entities.iter() {
        let obj = reference.get::<Spatial>();
        obj.set_transform(transform.to_godot_transform());
    }
}

fn pre_update_godot_transforms(
    mut entities: Query<(&mut Transform, &ErasedGodotRef)>,

    _godot_lock: GodotLock,
) {
    for (mut transform, reference) in entities.iter_mut() {
        let obj = reference.get::<Spatial>();
        *transform = obj.transform().to_bevy_transform();
    }
}

fn set_godot_transforms(
    entities: Query<(&Transform, &ErasedGodotRef), Added<ErasedGodotRef>>,
    _godot_lock: GodotLock,
) {
    for (transform, reference) in entities.iter() {
        reference
            .get::<Spatial>()
            .set_transform(transform.to_godot_transform());
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
