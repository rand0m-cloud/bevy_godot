use crate::AppState;
use bevy_godot::prelude::{
    bevy_prelude::{EventReader, State, SystemSet},
    *,
};

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::MainMenu).with_system(listen_for_play_button),
        )
        .add_system_set(SystemSet::on_pause(AppState::MainMenu).with_system(hide_main_menu))
        .add_startup_system(connect_play_button);
    }
}

fn connect_play_button(
    mut entities: Query<(&Name, &mut ErasedGodotRef)>,
    mut scene_tree: SceneTreeRef,
) {
    let mut play_button = entities
        .iter_mut()
        .find_map(|(name, reference)| {
            if name.as_str() == "StartButton" {
                Some(reference)
            } else {
                None
            }
        })
        .unwrap();

    connect_godot_signal(&mut play_button, "pressed", &mut scene_tree);
}

fn listen_for_play_button(
    mut events: EventReader<GodotSignal>,
    mut app_state: ResMut<State<AppState>>,
) {
    for evt in events.iter() {
        if evt.name() == "pressed" {
            app_state.push(AppState::InGame).unwrap();
        }
    }
}

fn hide_main_menu(mut entities: Query<(&Name, &mut ErasedGodotRef)>) {
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

    main_menu.get::<Control>().set_visible(false);
}
