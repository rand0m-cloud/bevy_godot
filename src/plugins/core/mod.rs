use crate::prelude::bevy_prelude::IntoSystem;
use bevy::app::*;
use iyes_loopless::condition::ConditionalSystemDescriptor;
use iyes_loopless::prelude::*;

pub mod godot_ref;
pub use godot_ref::*;

pub mod transforms;
pub use transforms::{Transform, Transform2D, *};

pub mod scene_tree;
pub use scene_tree::*;

pub mod collisions;
pub use collisions::*;

pub mod signals;
pub use signals::*;

pub struct GodotCorePlugin;

impl Plugin for GodotCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy::core::CorePlugin)
            .add_plugin(bevy::log::LogPlugin)
            .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
            .add_plugin(bevy::time::TimePlugin)
            .add_plugin(bevy::hierarchy::HierarchyPlugin)
            .add_plugin(GodotSceneTreePlugin)
            .add_plugin(GodotTransformsPlugin)
            .add_plugin(GodotCollisionsPlugin)
            .add_plugin(GodotSignalsPlugin);
    }
}

/// Bevy resource that is available when the app is updated through `_process` callback
pub struct GodotFrame;

/// Bevy resource that is available when the app is updated through `_physics_process` callback
pub struct GodotPhysicsFrame;

pub trait AsPhysicsSystem<Params> {
    fn as_physics_system(self) -> ConditionalSystemDescriptor;
}

impl<Params, T: IntoSystem<(), (), Params>> AsPhysicsSystem<Params> for T {
    fn as_physics_system(self) -> ConditionalSystemDescriptor {
        self.run_if_resource_exists::<GodotPhysicsFrame>()
    }
}
