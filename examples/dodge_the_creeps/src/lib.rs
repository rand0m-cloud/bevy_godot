use bevy_asset_loader::prelude::*;
use bevy_godot::prelude::*;

pub mod gameplay;
pub mod main_menu;
pub mod music;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::MainMenu),
        )
        .add_collection_to_loading_state::<_, music::MusicAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, main_menu::MenuAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, gameplay::enemy::EnemyAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, gameplay::player::PlayerAssets>(GameState::Loading)
        .init_resource::<Score>()
        .add_plugin(main_menu::MainMenuPlugin)
        .add_plugin(gameplay::GameplayPlugin)
        .add_plugin(music::MusicPlugin);
}

bevy_godot_init!(init, build_app);

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    MainMenu,
    Countdown,
    InGame,
    GameOver,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Resource)]
pub struct Score(i64);
