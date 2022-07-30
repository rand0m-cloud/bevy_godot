use crate::AppState;
use bevy_godot::prelude::{
    bevy_prelude::{State, SystemSet},
    *,
};

pub struct CountdownPlugin;
impl Plugin for CountdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Countdown)
                .with_system(setup_countdown)
                .with_system(kill_all_mobs),
        )
        .add_system_set(SystemSet::on_update(AppState::Countdown).with_system(update_countdown));
    }
}

pub struct CountdownTimer(Timer);

fn setup_countdown(mut commands: Commands) {
    commands.insert_resource(CountdownTimer(Timer::from_seconds(1.0, false)));
}

fn update_countdown(
    mut timer: ResMut<CountdownTimer>,
    time: Res<Time>,
    mut state: ResMut<State<AppState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        state.set(AppState::InGame).unwrap();
    }
}

fn kill_all_mobs(mut entities: Query<(&Groups, &mut ErasedGodotRef)>) {
    for (group, mut reference) in entities.iter_mut() {
        if group.is("mobs") {
            reference.get::<Node>().queue_free();
        }
    }
}
