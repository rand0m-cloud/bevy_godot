use crate::prelude::*;
use bevy::reflect::TypeUuid;
use gdnative::{
    object::{
        bounds::{RefImplBound, SafeAsRaw},
        ownership::Ownership,
    },
    prelude::{GodotObject, ManuallyManaged, Object, Ref, SubClass, TRef},
};

#[derive(Component, Reflect, Clone, Default, Debug, TypeUuid)]
#[reflect(Component)]
#[uuid = "1908221d-5b6a-4921-b831-4da0c2b8d182"]
pub struct ErasedGodotRef {
    object_id: i64,
    class_name: String,
}

impl ErasedGodotRef {
    pub fn get<T: GodotObject<Memory = ManuallyManaged>>(&mut self) -> TRef<T> {
        self.try_get()
            .unwrap_or_else(|| panic!("failed to get godot ref as {}", std::any::type_name::<T>()))
    }

    pub fn try_get<T: GodotObject<Memory = ManuallyManaged>>(&mut self) -> Option<TRef<T>> {
        // SAFETY: The caller must uphold the contract of the constructors to ensure exclusive access
        unsafe { TRef::try_from_instance_id(self.object_id) }
    }

    /// # Safety
    /// When using ErasedGodotRef as a Bevy Resource or Component, do not create duplicate references to the same instance because Godot is not completely thread-safe.
    pub unsafe fn new<T: GodotObject<Memory = ManuallyManaged> + SubClass<Object>, Own: Ownership>(
        reference: Ref<T, Own>,
    ) -> Self
    where
        RefImplBound: SafeAsRaw<ManuallyManaged, Own>,
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
