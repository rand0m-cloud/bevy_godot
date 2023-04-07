use crate::GameState;
use bevy_asset_loader::prelude::*;
use bevy_godot::prelude::*;

#[derive(AssetCollection, Resource, Debug)]
pub struct MenuAssets {
    #[asset]
    pub menu_label: Handle<ErasedGodotRef>,

    #[asset]
    pub play_button: Handle<ErasedGodotRef>,
}

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                init_menu_assets,
                connect_play_button.after(init_menu_assets),
            )
                .in_schedule(OnExit(GameState::Loading)),
        )
        .add_system(listen_for_play_button.in_set(OnUpdate(GameState::MainMenu)));

        // Not clear what "on_pause" and "on_resume" look like in bevy 0.10 - or what is triggering
        // these states in this example in bevy 0.9
        // .add_system_set(SystemSet::on_pause(GameState::MainMenu).with_system(hide_play_button))
        // .add_system_set(SystemSet::on_resume(GameState::MainMenu).with_system(show_play_button));
    }
}

#[derive(NodeTreeView)]
pub struct MenuUi {
    #[node("Main/CanvasLayer/HUD/MainMenu/MessageLabel")]
    menu_label: ErasedGodotRef,

    #[node("Main/CanvasLayer/HUD/MainMenu/StartButton")]
    play_button: ErasedGodotRef,
}

fn init_menu_assets(
    mut menu_assets: ResMut<MenuAssets>,
    mut assets: ResMut<Assets<ErasedGodotRef>>,
    mut scene_tree: SceneTreeRef,
) {
    let menu_ui = MenuUi::from_node(scene_tree.get_root());

    menu_assets.menu_label = assets.add(menu_ui.menu_label);
    menu_assets.play_button = assets.add(menu_ui.play_button);
}

fn connect_play_button(
    menu_assets: Res<MenuAssets>,
    mut assets: ResMut<Assets<ErasedGodotRef>>,
    mut scene_tree: SceneTreeRef,
) {
    let play_button = assets.get_mut(&menu_assets.play_button).unwrap();
    connect_godot_signal(play_button, "pressed", &mut scene_tree);
}

fn listen_for_play_button(
    mut events: EventReader<GodotSignal>,
    mut app_state: ResMut<NextState<GameState>>,
) {
    for evt in events.iter() {
        if evt.name() == "pressed" {
            app_state.set(GameState::Countdown);
        }
    }
}

#[allow(dead_code)]
fn hide_play_button(menu_assets: Res<MenuAssets>, mut assets: ResMut<Assets<ErasedGodotRef>>) {
    assets
        .get_mut(&menu_assets.play_button)
        .unwrap()
        .get::<Control>()
        .set_visible(false);
}

#[allow(dead_code)]
fn show_play_button(menu_assets: Res<MenuAssets>, mut assets: ResMut<Assets<ErasedGodotRef>>) {
    assets
        .get_mut(&menu_assets.play_button)
        .unwrap()
        .get::<Control>()
        .set_visible(true);
}
