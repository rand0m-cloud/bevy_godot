use bevy::{
    app::*, asset::AssetPlugin, ecs::schedule::IntoSystemDescriptor, input::InputPlugin,
    prelude::*, scene::ScenePlugin, window::WindowPlugin,
};
pub mod godot_ref;
pub use godot_ref::*;

pub mod transforms;
pub use transforms::{Transform, *};

pub mod scene_tree;
pub use scene_tree::*;

#[derive(StageLabel, Clone, Hash, Debug, PartialEq, Eq)]
pub enum GodotStage {
    SceneTreeUpdate,
    BeforeBevy,
    GodotUpdate,
    AfterBevy,
}

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
        .add_stage_before(
            CoreStage::First,
            GodotStage::SceneTreeUpdate,
            SystemStage::parallel(),
        )
        .add_stage_after(
            GodotStage::SceneTreeUpdate,
            GodotStage::BeforeBevy,
            SystemStage::parallel(),
        )
        .add_stage_after(
            CoreStage::Update,
            GodotStage::GodotUpdate,
            SystemStage::parallel(),
        )
        .add_stage_after(
            CoreStage::Last,
            GodotStage::AfterBevy,
            SystemStage::parallel(),
        )
        .add_plugin(GodotSceneTreePlugin)
        .add_plugin(GodotTransformsPlugin);
    }
}

pub trait GodotBevyExt {
    fn add_godot_system<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut App;
}

impl GodotBevyExt for App {
    fn add_godot_system<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut App {
        self.add_system_to_stage(GodotStage::GodotUpdate, system)
    }
}
