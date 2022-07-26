#![allow(clippy::type_complexity)]

use bevy::core::Timer;
use bevy_godot::prelude::*;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_system(print_ball_positions)
        .add_system(print_ball_collisions)
        .insert_resource(PrintEntitiesTimer(Timer::from_seconds(0.5, true)));
}

bevy_godot_init!(init, build_app);

pub struct PrintEntitiesTimer(pub Timer);

fn print_ball_positions(
    entities: Query<(&Name, &Groups, &Transform)>,
    _scene_tree: SceneTreeRef,
    time: Res<Time>,
    mut print_timer: ResMut<PrintEntitiesTimer>,
) {
    print_timer.0.tick(time.delta());
    if !print_timer.0.just_finished() {
        return;
    }

    for (name, groups, transform) in entities.iter() {
        if groups.is("ball") {
            println!("{} has origin of {}", name, transform.as_bevy().translation);
        }
    }
}

fn print_ball_collisions(
    entities: Query<(&Name, &Groups, &Collisions)>,
    all_entities: Query<&Name>,
) {
    for (name, groups, collisions) in entities.iter() {
        if !groups.is("ball") {
            continue;
        }

        if collisions.recent_collisions().len() > 0 {
            for other in collisions.recent_collisions() {
                println!(
                    "{} collided with {}",
                    name,
                    all_entities.get(*other).unwrap()
                );
            }
        }
    }
}
