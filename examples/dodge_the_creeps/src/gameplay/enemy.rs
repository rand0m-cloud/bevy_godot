use crate::GameState;
use bevy_asset_loader::prelude::*;
use bevy_godot::prelude::{
    godot_prelude::{AnimatedSprite, PathFollow2D, RigidBody2D},
    *,
};
use std::f64::consts::PI;

#[derive(AssetCollection, Debug)]
pub struct EnemyAssets {
    #[asset(path = "Mob.tscn")]
    mob_scn: Handle<GodotResource>,
}

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
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
    mut entities: Query<(&Name, &mut ErasedGodotRef)>,
    assets: Res<EnemyAssets>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let mut mob_spawn_path_follow = entities
        .iter_mut()
        .find_map(|(name, reference)| (name.as_str() == "MobSpawnLocation").then_some(reference))
        .unwrap();
    let mob_spawn_path_follow = mob_spawn_path_follow.get::<PathFollow2D>();

    mob_spawn_path_follow.set_offset(fastrand::i32(..) as f64);

    let mut direction = mob_spawn_path_follow.rotation() + PI / 2.0;
    direction += fastrand::f64() * PI / 2.0 - PI / 4.0;

    let position = mob_spawn_path_follow.position();

    let mut transform = GodotTransform2D::IDENTITY.translated(position);
    transform.set_rotation(direction as f32);

    commands
        .spawn()
        .insert(Mob { direction })
        .insert(Transform2D::from(transform))
        .insert(GodotScene::from_handle(&assets.mob_scn));
}

#[derive(NodeTreeView)]
pub struct MobNodes {
    #[node("AnimatedSprite")]
    animated_sprite: ErasedGodotRef,

    #[node("VisibilityNotifier2D")]
    visibility_notifier: ErasedGodotRef,
}

fn new_mob(
    mut entities: Query<(&Mob, &mut ErasedGodotRef), Added<Mob>>,
    mut scene_tree: SceneTreeRef,
) {
    for (mob_data, mut mob) in entities.iter_mut() {
        let mob = mob.get::<RigidBody2D>();

        let velocity = Vector2::new(fastrand::f32() * 100.0 + 150.0, 0.0);
        mob.set_linear_velocity(velocity.rotated(mob_data.direction as f32));

        let mut mob_nodes = MobNodes::from_node(mob);

        let animated_sprite = mob_nodes.animated_sprite.get::<AnimatedSprite>();

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

        connect_godot_signal(
            &mut mob_nodes.visibility_notifier,
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
