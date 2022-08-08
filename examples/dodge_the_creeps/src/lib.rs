use bevy_asset_loader::prelude::*;
use bevy_godot::prelude::*;
pub mod gameplay;
pub mod main_menu;
pub mod music;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_state(GameState::Loading)
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::MainMenu)
                .with_collection::<music::MusicAssets>()
                .with_collection::<main_menu::MenuAssets>()
                .with_collection::<gameplay::enemy::EnemyAssets>()
                .with_collection::<gameplay::player::PlayerAssets>(),
        )
        .init_resource::<Score>()
        .add_plugin(main_menu::MainMenuPlugin)
        .add_plugin(gameplay::GameplayPlugin)
        .add_plugin(music::MusicPlugin);
}

bevy_godot_init!(init, build_app);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Loading,
    MainMenu,
    Countdown,
    InGame,
    GameOver,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Score(i64);
