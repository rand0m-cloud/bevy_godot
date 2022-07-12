use crate::prelude::*;
use gdnative::{
    api::{Object, Reference},
    object::{
        bounds::{RefImplBound, SafeAsRaw},
        memory::RefCounted,
        ownership::Ownership,
        GodotObject, Ref, SubClass, TRef,
    },
};
use std::marker::PhantomData;

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct GodotReference<T: GodotObject + 'static> {
    #[reflect(ignore)]
    object: Option<Ref<Reference>>,
    #[reflect(ignore)]
    phantom: PhantomData<Ref<T>>,
}

impl<T: GodotObject + 'static> Default for GodotReference<T> {
    fn default() -> Self {
        Self {
            object: None,
            phantom: PhantomData,
        }
    }
}

impl<T: GodotObject<Memory = RefCounted> + SubClass<Reference>> GodotReference<T> {
    pub fn get(&self) -> TRef<T> {
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
    object_id: i64,
    class_name: String,
}

impl ErasedGodotRef {
    pub fn get<T: GodotObject>(&mut self) -> TRef<T> {
        self.try_get()
            .unwrap_or_else(|| panic!("failed to get godot ref as {}", std::any::type_name::<T>()))
    }

    pub fn try_get<T: GodotObject>(&mut self) -> Option<TRef<T>> {
        // SAFETY: The caller must uphold the contract of the constructors to ensure exclusive access
        unsafe { TRef::try_from_instance_id(self.object_id) }
    }

    /// # Safety
    /// When using ErasedGodotRef as a Bevy Resource or Component, do not create duplicate references to the same instance because Godot is not completely thread-safe.
    pub unsafe fn new<T: GodotObject + SubClass<Object>, Own: Ownership>(
        reference: Ref<T, Own>,
    ) -> Self
    where
        RefImplBound: SafeAsRaw<<T as GodotObject>::Memory, Own>,
    {
        let obj = Object::cast_ref(reference.as_raw().cast().unwrap());
        Self::from_instance_id(obj.get_instance_id())
    }

    pub fn instance_id(&self) -> i64 {
        self.object_id
    }

    /// # Safety
    /// Look to [Self::new]
    pub unsafe fn from_instance_id(id: i64) -> Self {
        let obj: TRef<Object> = TRef::from_instance_id(id);
        let object_id = obj.get_instance_id();
        let class_name = obj.get_class().to_string();
        Self {
            object_id,
            class_name,
        }
    }
}
