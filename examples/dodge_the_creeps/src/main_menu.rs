use crate::GameState;
use bevy_asset_loader::prelude::*;
use bevy_godot::prelude::*;

#[derive(AssetCollection, Debug)]
pub struct MenuAssets {
    #[asset]
    pub menu_label: Handle<ErasedGodotRef>,

    #[asset]
    pub play_button: Handle<ErasedGodotRef>,
}

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_exit(GameState::Loading)
                .with_system(init_menu_assets)
                .with_system(connect_play_button.after(init_menu_assets)),
        )
        .add_system_set(
            SystemSet::on_update(GameState::MainMenu).with_system(listen_for_play_button),
        )
        .add_system_set(SystemSet::on_pause(GameState::MainMenu).with_system(hide_play_button))
        .add_system_set(SystemSet::on_resume(GameState::MainMenu).with_system(show_play_button));
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
    mut app_state: ResMut<State<GameState>>,
) {
    for evt in events.iter() {
        if evt.name() == "pressed" {
            app_state.push(GameState::Countdown).unwrap();
        }
    }
}

fn hide_play_button(menu_assets: Res<MenuAssets>, mut assets: ResMut<Assets<ErasedGodotRef>>) {
    assets
        .get_mut(&menu_assets.play_button)
        .unwrap()
        .get::<Control>()
        .set_visible(false);
}

fn show_play_button(menu_assets: Res<MenuAssets>, mut assets: ResMut<Assets<ErasedGodotRef>>) {
    assets
        .get_mut(&menu_assets.play_button)
        .unwrap()
        .get::<Control>()
        .set_visible(true);
}
