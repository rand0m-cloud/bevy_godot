pub mod assets;
pub mod core;
pub mod packed_scene;

use bevy::app::*;

pub struct DefaultGodotPlugin;

impl Plugin for DefaultGodotPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(core::GodotCorePlugin)
            .add_plugin(packed_scene::PackedScenePlugin)
            .add_plugin(assets::GodotAssetsPlugin);
    }
}
