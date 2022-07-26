use bevy::app::*;

pub mod godot_ref;
pub use godot_ref::*;

pub mod transforms;
pub use transforms::{Transform, *};

pub mod scene_tree;
pub use scene_tree::*;

pub mod collisions;
pub use collisions::*;

pub struct GodotCorePlugin;

impl Plugin for GodotCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy::core::CorePlugin)
            .add_plugin(bevy::log::LogPlugin)
            .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
            .add_plugin(GodotSceneTreePlugin)
            .add_plugin(GodotTransformsPlugin)
            .add_plugin(GodotCollisionsPlugin);
    }
}
