use bevy_asset_loader::prelude::*;
use bevy_godot::prelude::*;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Playing),
        )
        .add_collection_to_loading_state::<_, GameAssets>(GameState::Loading)
        .add_system(spawn_cube_asset.in_schedule(OnEnter(GameState::Playing)));
}

bevy_godot_init!(init, build_app);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}

#[derive(AssetCollection, Resource, Debug)]
struct GameAssets {
    #[asset(path = "simple_scene.tscn")]
    player: Handle<GodotResource>,
}

fn spawn_cube_asset(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    entities: Query<(&Name, &Transform)>,
) {
    let spawn_location = entities
        .iter()
        .find_entity_by_name("SpawnPosition")
        .unwrap();

    commands
        .spawn_empty()
        .insert(*spawn_location)
        .insert(GodotScene::from_handle(&game_assets.player));
}
