pub use crate::plugins::{assets::GodotResource, core::*, packed_scene::*};

pub mod bevy_prelude {
    pub use bevy::prelude::*;
}
pub use bevy;

pub mod godot_prelude {
    pub use gdnative::api;
    pub use gdnative::prelude::*;
}
pub use gdnative;

pub use bevy_prelude::{
    AddAsset, App, AssetServer, Assets, Children, Commands, Component, DespawnRecursiveExt, Entity,
    Handle, HandleUntyped, Name, ParallelSystemDescriptorCoercion, Parent, Plugin, Query, Reflect,
    ReflectComponent, Res, ResMut, SystemSet, Time, Timer, Transform as BevyTransform, Vec3, World,
};
pub use godot_prelude::{
    api::*, godot_init, methods, GodotObject, InitHandle, NativeClass, Ref, TRef,
    Transform as GodotTransform, Transform2D as GodotTransform2D,
};

pub use crate::bevy_godot_init;
pub use crate::GodotPlugin;
