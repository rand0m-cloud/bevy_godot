use crate::AppState;
use bevy_godot::prelude::{
    bevy_prelude::{Local, SystemSet, With},
    godot_prelude::Vector2,
    *,
};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(player_on_ready)
            .add_system_set(SystemSet::on_update(AppState::InGame).with_system(move_player))
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_player));
    }
}

#[derive(Debug, Component)]
pub struct Player {
    speed: f64,
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn()
        .insert(GodotScene::from_path("res://Player.tscn"))
        .insert(Player { speed: 400.0 });
}

fn player_on_ready(
    mut player: Query<&mut ErasedGodotRef, With<Player>>,
    mut complete: Local<bool>,
    _scene_tree: SceneTreeRef,
) {
    if *complete {
        return;
    }

    if let Ok(mut player) = player.get_single_mut() {
        let player = player.get::<Node2D>();

        player.set_visible(false);

        let start_position = unsafe {
            player
                .get_node("/root/Main/StartPosition")
                .unwrap()
                .assume_safe()
                .cast::<Node2D>()
                .unwrap()
        };
        player.set_position(start_position.position());

        *complete = true;
    }
}

fn move_player(
    mut player: Query<(&Player, &mut ErasedGodotRef, &mut Transform2D)>,
    time: Res<Time>,
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

    transform.origin += velocity * time.delta_seconds();
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
    _scene_tree: SceneTreeRef,
) {
    let (mut player, mut transform) = player.single_mut();
    let player = player.get::<Node2D>();

    player.set_visible(true);

    let start_position = unsafe {
        player
            .get_node("/root/Main/StartPosition")
            .unwrap()
            .assume_safe()
            .cast::<Node2D>()
            .unwrap()
    };
    transform.origin = start_position.position();
}
