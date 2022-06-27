pub mod core;
pub mod packed_scene;
pub mod prelude;

use bevy::app::*;

pub struct DefaultGodotPlugin;

impl Plugin for DefaultGodotPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(core::GodotCorePlugin)
            .add_plugin(packed_scene::PackedScenePlugin);
    }
}
