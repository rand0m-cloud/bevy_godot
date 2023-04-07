use crate::main_menu::MenuAssets;
use crate::GameState;
use bevy_godot::prelude::{godot_prelude::Label, *};

pub struct GameoverPlugin;
impl Plugin for GameoverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_gameover.in_schedule(OnEnter(GameState::GameOver)))
            .add_system(update_gameover_timer.in_set(OnUpdate(GameState::GameOver)));
    }
}

#[derive(Resource)]
pub struct GameoverTimer(Timer);

fn setup_gameover(
    mut commands: Commands,

    menu_assets: Res<MenuAssets>,
    mut assets: ResMut<Assets<ErasedGodotRef>>,
) {
    commands.insert_resource(GameoverTimer(Timer::from_seconds(2.0, TimerMode::Once)));

    assets
        .get_mut(&menu_assets.menu_label)
        .unwrap()
        .get::<Label>()
        .set_text("Game Over");
}

fn update_gameover_timer(
    mut timer: ResMut<GameoverTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,

    menu_assets: Res<MenuAssets>,
    mut assets: ResMut<Assets<ErasedGodotRef>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    next_state.set(GameState::MainMenu);

    assets
        .get_mut(&menu_assets.menu_label)
        .unwrap()
        .get::<Label>()
        .set_text("Dodge the\nCreeps");
}
