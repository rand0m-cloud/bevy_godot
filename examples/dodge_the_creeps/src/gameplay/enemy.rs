use crate::AppState;
use bevy_godot::prelude::{
    bevy_prelude::{Added, EventReader, SystemSet},
    godot_prelude::Vector2,
    *,
};
use std::f64::consts::PI;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(spawn_mob)
                .with_system(new_mob)
                .with_system(kill_mob),
        )
        .insert_resource(MobSpawnTimer(Timer::from_seconds(0.5, true)));
    }
}

#[derive(Debug, Component)]
pub struct Mob {
    direction: f64,
}

pub struct MobSpawnTimer(Timer);

fn spawn_mob(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<MobSpawnTimer>,
    mut scene_tree: SceneTreeRef,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let scene_root = scene_tree.get().root().unwrap();
    let mob_spawn_location = unsafe {
        scene_root
            .assume_safe()
            .get_node("Main/MobPath/MobSpawnLocation")
            .unwrap()
            .assume_safe()
            .cast::<PathFollow2D>()
            .unwrap()
    };
    mob_spawn_location.set_offset(fastrand::i32(..) as f64);

    let mut direction = mob_spawn_location.rotation() + PI / 2.0;
    direction += fastrand::f64() * PI / 2.0 - PI / 4.0;

    let position = mob_spawn_location.position();

    let mut transform = GodotTransform2D::IDENTITY.translated(position);
    transform.set_rotation(direction as f32);

    commands
        .spawn()
        .insert(Mob { direction })
        .insert(Transform2D::from(transform))
        .insert(GodotScene::from_path("res://Mob.tscn"));
}

fn new_mob(
    mut entities: Query<(&Mob, &mut ErasedGodotRef), Added<Mob>>,
    mut scene_tree: SceneTreeRef,
) {
    for (mob_data, mut mob) in entities.iter_mut() {
        let mob = mob.get::<RigidBody2D>();

        let velocity = Vector2::new(fastrand::f32() * 100.0 + 150.0, 0.0);
        mob.set_linear_velocity(velocity.rotated(mob_data.direction as f32));

        let animated_sprite = unsafe {
            mob.get_node("AnimatedSprite")
                .unwrap()
                .assume_safe()
                .cast::<AnimatedSprite>()
                .unwrap()
        };

        animated_sprite.play("", false);

        let mob_types = unsafe {
            animated_sprite
                .sprite_frames()
                .unwrap()
                .assume_safe()
                .get_animation_names()
        };
        let mob_type_index = fastrand::i32(0..mob_types.len());
        animated_sprite.set_animation(mob_types.get(mob_type_index));

        let visibility_notifier = mob.get_node("VisibilityNotifier2D").unwrap();

        connect_godot_signal(
            &mut unsafe { ErasedGodotRef::new(visibility_notifier.assume_unique()) },
            "screen_exited",
            &mut scene_tree,
        );
    }
}

fn kill_mob(mut signals: EventReader<GodotSignal>) {
    for signal in signals.iter() {
        if signal.name() == "screen_exited" {
            unsafe {
                signal
                    .origin()
                    .get::<Node>()
                    .get_parent()
                    .unwrap()
                    .assume_safe()
                    .queue_free();
            }
        }
    }
}
