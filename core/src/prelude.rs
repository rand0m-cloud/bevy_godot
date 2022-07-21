pub use crate::plugins::{core::*, packed_scene::*};

pub mod bevy_prelude {
    pub use bevy::prelude::*;
}
pub mod godot_prelude {
    pub use gdnative::api;
    pub use gdnative::prelude::*;
}

pub use bevy_prelude::{
    App, Children, Commands, Component, DespawnRecursiveExt, Entity, Name,
    ParallelSystemDescriptorCoercion, Parent, Plugin, Query, Reflect, ReflectComponent, Res,
    ResMut, Time, Timer, Transform as BevyTransform, Vec3,
};
pub use godot_prelude::{
    api::*, godot_init, methods, GodotObject, InitHandle, NativeClass, Ref, TRef,
    Transform as GodotTransform,
};

pub use crate::bevy_godot_init;
pub use crate::GodotPlugin;
