use crate::main_menu::MenuAssets;
use crate::GameState;
use bevy_godot::prelude::{godot_prelude::Label, *};

pub struct CountdownPlugin;
impl Plugin for CountdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Countdown)
                .with_system(setup_countdown)
                .with_system(kill_all_mobs),
        )
        .add_system_set(SystemSet::on_update(GameState::Countdown).with_system(update_countdown));
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
    mut state: ResMut<State<GameState>>,

    menu_assets: Res<MenuAssets>,
    mut assets: ResMut<Assets<ErasedGodotRef>>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        state.set(GameState::InGame).unwrap();

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
