use bevy::{app::*, prelude::*};
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
use std::marker::PhantomData;

pub struct GodotCorePlugin;

impl Plugin for GodotCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy::core::CorePlugin)
            .add_system_to_stage(CoreStage::PostUpdate, post_update_godot_transforms)
            .add_system_to_stage(CoreStage::PreUpdate, pre_update_godot_transforms)
            .add_system_to_stage(CoreStage::PostUpdate, set_godot_transforms)
            .insert_non_send_resource(GodotLockImpl)
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

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct ErasedGodotRef {
    #[reflect(ignore)]
    object_id: i64,
    class_name: String,
}

impl ErasedGodotRef {
    pub fn get<T: GodotObject>(&self) -> TRef<'_, T> {
        self.try_get().unwrap()
    }

    pub fn try_get<T: GodotObject>(&self) -> Option<TRef<'_, T>> {
        unsafe { TRef::try_from_instance_id(self.object_id) }
    }

    pub fn new<T: GodotObject + SubClass<Object>, Own: Ownership>(reference: Ref<T, Own>) -> Self
    where
        RefImplBound: SafeAsRaw<<T as GodotObject>::Memory, Own>,
    {
        let obj = Object::cast_ref(reference.as_raw().cast().unwrap());
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
