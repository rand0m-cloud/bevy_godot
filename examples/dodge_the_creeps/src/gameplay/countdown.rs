use crate::main_menu::MenuAssets;
use crate::GameState;
use bevy_godot::prelude::{godot_prelude::Label, *};

pub struct CountdownPlugin;
impl Plugin for CountdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (setup_countdown, kill_all_mobs).in_schedule(OnEnter(GameState::Countdown)),
        )
        .add_system(update_countdown.in_set(OnUpdate(GameState::Countdown)));
    }
}

#[derive(Resource)]
pub struct CountdownTimer(Timer);

fn setup_countdown(
    mut commands: Commands,

    menu_assets: Res<MenuAssets>,
    mut assets: ResMut<Assets<ErasedGodotRef>>,
) {
    commands.insert_resource(CountdownTimer(Timer::from_seconds(1.0, TimerMode::Once)));

    assets
        .get_mut(&menu_assets.menu_label)
        .unwrap()
        .get::<Label>()
        .set_text("Get Ready");
}

fn update_countdown(
    mut timer: ResMut<CountdownTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,

    menu_assets: Res<MenuAssets>,
    mut assets: ResMut<Assets<ErasedGodotRef>>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        next_state.set(GameState::InGame);

        assets
            .get_mut(&menu_assets.menu_label)
            .unwrap()
            .get::<Label>()
            .set_text("");
    }
}

fn kill_all_mobs(mut entities: Query<(&Groups, &mut ErasedGodotRef)>) {
    for (group, mut reference) in entities.iter_mut() {
        if group.is("mobs") {
            reference.get::<Node>().queue_free();
        }
    }
}
