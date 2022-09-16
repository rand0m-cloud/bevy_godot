#![allow(clippy::type_complexity)]
use bevy::app::*;

pub mod node_tree_view;
pub mod plugins;
pub mod prelude;

pub mod init_macro;
pub use init_macro::*;

pub struct GodotPlugin;

impl Plugin for GodotPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(plugins::DefaultGodotPlugin);
    }
}
