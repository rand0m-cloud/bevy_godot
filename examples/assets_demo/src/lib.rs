use bevy_asset_loader::*;
use bevy_godot::prelude::*;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    AssetLoader::new(GameState::Loading)
        .with_collection::<GameAssets>()
        .continue_to_state(GameState::Playing)
        .build(app);

    app.add_state(GameState::Loading)
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_cube_asset));
}

bevy_godot_init!(init, build_app);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    Loading,
    Playing,
}

#[derive(AssetCollection, Debug)]
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
        .find_map(|(name, transform)| (name.as_str() == "SpawnPosition").then_some(*transform))
        .unwrap();

    commands
        .spawn()
        .insert(spawn_location)
        .insert(GodotScene::from_handle(&game_assets.player));
}
