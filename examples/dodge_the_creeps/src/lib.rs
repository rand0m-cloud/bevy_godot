use bevy_godot::prelude::*;
mod gameplay;
mod main_menu;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_state(AppState::MainMenu)
        .init_resource::<Score>()
        .add_plugin(main_menu::MainMenuPlugin)
        .add_plugin(gameplay::GameplayPlugin);
}

bevy_godot_init!(init, build_app);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MainMenu,
    InGame,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Score(i64);
