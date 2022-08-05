use bevy_asset_loader::*;
use bevy_godot::prelude::*;
pub mod gameplay;
pub mod main_menu;
pub mod music;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    AssetLoader::new(AppState::Loading)
        .with_collection::<music::MusicAssets>()
        .continue_to_state(AppState::MainMenu)
        .build(app);

    app.add_state(AppState::Loading)
        .init_resource::<Score>()
        .add_plugin(main_menu::MainMenuPlugin)
        .add_plugin(gameplay::GameplayPlugin)
        .add_plugin(music::MusicPlugin);
}

bevy_godot_init!(init, build_app);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Loading,
    MainMenu,
    Countdown,
    InGame,
    GameOver,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Score(i64);
