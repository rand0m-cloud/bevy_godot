use bevy::{
    app::*, asset::AssetPlugin, input::InputPlugin, prelude::*, scene::ScenePlugin,
    window::WindowPlugin,
};
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
        app.add_plugins_with(DefaultPlugins, |group| {
            group
                .disable::<InputPlugin>()
                .disable::<WindowPlugin>()
                .disable::<AssetPlugin>()
                .disable::<ScenePlugin>();

            #[cfg(feature = "trace")]
            group.disable::<bevy::render::RenderPlugin>();

            group
        })
        .add_plugin(GodotSceneTreePlugin)
        .add_plugin(GodotTransformsPlugin)
        .add_plugin(GodotCollisionsPlugin);
    }
}
