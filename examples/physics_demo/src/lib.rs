#![allow(clippy::type_complexity)]

use bevy::core::Timer;
use bevy_godot::prelude::*;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_system(print_balls_position)
        .insert_resource(PrintEntitiesTimer(Timer::from_seconds(0.1, true)));
}

bevy_godot_init!(init, build_app);

pub struct PrintEntitiesTimer(pub Timer);

fn print_balls_position(
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
