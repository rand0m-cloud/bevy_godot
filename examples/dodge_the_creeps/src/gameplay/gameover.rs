use crate::main_menu::MenuAssets;
use crate::GameState;
use bevy_godot::prelude::{godot_prelude::Label, *};

pub struct GameoverPlugin;
impl Plugin for GameoverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(setup_gameover))
            .add_system_set(
                SystemSet::on_update(GameState::GameOver).with_system(update_gameover_timer),
            );
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
    mut state: ResMut<State<GameState>>,

    menu_assets: Res<MenuAssets>,
    mut assets: ResMut<Assets<ErasedGodotRef>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    state.pop().unwrap();

    assets
        .get_mut(&menu_assets.menu_label)
        .unwrap()
        .get::<Label>()
        .set_text("Dodge the\nCreeps");
}
