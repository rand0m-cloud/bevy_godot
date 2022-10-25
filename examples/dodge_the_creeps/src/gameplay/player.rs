use crate::GameState;
use bevy_asset_loader::prelude::*;
use bevy_godot::prelude::{
    godot_prelude::{AnimatedSprite, Input},
    *,
};

#[derive(AssetCollection, Debug)]
pub struct PlayerAssets {
    #[asset(path = "Player.tscn")]
    player_scn: Handle<GodotResource>,
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_exit(GameState::Loading).with_system(spawn_player))
            .add_system(player_on_ready)
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .with_system(move_player.as_physics_system())
                    .with_system(check_player_death),
            )
            .add_system_set(SystemSet::on_enter(GameState::Countdown).with_system(setup_player))
            .add_system_set(
                SystemSet::on_update(GameState::Countdown)
                    .with_system(move_player.as_physics_system()),
            );
    }
}

#[derive(Debug, Component)]
pub struct Player {
    speed: f64,
}

fn spawn_player(mut commands: Commands, assets: Res<PlayerAssets>) {
    commands
        .spawn()
        .insert(GodotScene::from_handle(&assets.player_scn))
        .insert(Player { speed: 400.0 });
}

#[derive(NodeTreeView)]
pub struct PlayerStartPosition(#[node("/root/Main/StartPosition")] ErasedGodotRef);

fn player_on_ready(mut player: Query<&mut ErasedGodotRef, Added<Player>>) {
    if let Ok(mut player) = player.get_single_mut() {
        let player = player.get::<Node2D>();

        player.set_visible(false);

        let mut start_position = PlayerStartPosition::from_node(player);
        player.set_position(start_position.0.get::<Node2D>().position());
    }
}

fn move_player(
    mut player: Query<(&Player, &mut ErasedGodotRef, &mut Transform2D)>,
    mut system_delta: SystemDeltaTimer,
) {
    let (player, mut player_ref, mut transform) = player.single_mut();

    let screen_size = player_ref.get::<Node2D>().get_viewport_rect().size;

    let animated_sprite = unsafe {
        player_ref
            .get::<Node>()
            .get_node("AnimatedSprite")
            .unwrap()
            .assume_safe()
            .cast::<AnimatedSprite>()
            .unwrap()
    };

    let input = Input::godot_singleton();
    let input_dir = input.get_vector("move_left", "move_right", "move_up", "move_down", -1.0);

    let velocity = if input_dir.length() > 0.0 {
        animated_sprite.play("", false);
        input_dir.normalized() * player.speed as f32
    } else {
        animated_sprite.stop();
        Vector2::ZERO
    };

    transform.origin += velocity * system_delta.delta_seconds();
    transform.origin.x = f32::min(f32::max(0.0, transform.origin.x), screen_size.x);
    transform.origin.y = f32::min(f32::max(0.0, transform.origin.y), screen_size.y);

    if velocity.x != 0.0 {
        animated_sprite.set_animation("right");
        animated_sprite.set_flip_v(false);
        animated_sprite.set_flip_h(velocity.x < 0.0);
    } else if velocity.y != 0.0 {
        animated_sprite.set_animation("up");
        animated_sprite.set_flip_v(velocity.y > 0.0);
    }
}

fn setup_player(
    mut player: Query<(&mut ErasedGodotRef, &mut Transform2D), With<Player>>,
    mut entities: Query<(&Name, &mut ErasedGodotRef), Without<Player>>,
) {
    let (mut player, mut transform) = player.single_mut();
    let player = player.get::<Node2D>();

    player.set_visible(true);

    let start_position = entities
        .iter_mut()
        .find_map(|(name, reference)| (name.as_str() == "StartPosition").then_some(reference))
        .unwrap()
        .get::<Node2D>()
        .position();
    transform.origin = start_position;
}

fn check_player_death(
    mut player: Query<(&mut ErasedGodotRef, &Collisions), With<Player>>,
    mut state: ResMut<State<GameState>>,
) {
    let (mut player_ref, collisions) = player.single_mut();

    if collisions.colliding().is_empty() {
        return;
    }

    player_ref.get::<Node2D>().set_visible(false);
    state.set(GameState::GameOver).unwrap();
}
