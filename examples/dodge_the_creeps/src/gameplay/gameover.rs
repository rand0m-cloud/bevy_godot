use crate::AppState;
use bevy_godot::prelude::{
    bevy_prelude::{State, SystemSet},
    *,
};

pub struct GameoverPlugin;
impl Plugin for GameoverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::GameOver).with_system(setup_gameover_button_timer),
        )
        .add_system_set(
            SystemSet::on_update(AppState::GameOver).with_system(update_gameover_button_timer),
        );
    }
}

pub struct GameoverButtonTimer(Timer);

fn setup_gameover_button_timer(mut commands: Commands) {
    commands.insert_resource(GameoverButtonTimer(Timer::from_seconds(2.0, false)));
}

fn update_gameover_button_timer(
    mut timer: ResMut<GameoverButtonTimer>,
    time: Res<Time>,
    mut entities: Query<(&Name, &mut ErasedGodotRef)>,
    mut state: ResMut<State<AppState>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let mut main_menu = entities
        .iter_mut()
        .find_map(|(name, reference)| {
            if name.as_str() == "MainMenu" {
                Some(reference)
            } else {
                None
            }
        })
        .unwrap();
    main_menu.get::<Control>().set_visible(true);

    state.set(AppState::MainMenu).unwrap();
}
