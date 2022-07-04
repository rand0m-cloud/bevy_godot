use crate::prelude::*;
use gdnative::{
    api::{Object, Reference},
    object::{
        bounds::{RefImplBound, SafeAsRaw},
        memory::{ManuallyManaged, RefCounted},
        ownership::Ownership,
        GodotObject, Ref, SubClass, TRef,
    },
};
use std::marker::PhantomData;

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
