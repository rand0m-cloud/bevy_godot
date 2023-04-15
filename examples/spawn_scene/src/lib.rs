use bevy_godot::prelude::*;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_startup_system(spawn_cube).add_system(move_cubes);
}

bevy_godot_init!(init, build_app);

#[derive(Component)]
pub struct Cube {
    starting_position: Vec3,
}

fn spawn_cube(mut commands: Commands) {
    for x in [-3.0, 0.0, 3.0] {
        let starting_position = Vec3::new(x, 0.0, -5.0);
        commands
            .spawn_empty()
            .insert(GodotScene::from_path("res://simple_scene.tscn"))
            .insert(Cube { starting_position })
            .insert(Transform::from(BevyTransform::from_translation(
                starting_position,
            )));
    }
}

fn move_cubes(mut cubes: Query<(&Cube, &mut Transform)>, time: Res<Time>) {
    for (cube, mut transform) in cubes.iter_mut() {
        transform.as_bevy_mut().translation =
            5.0 * time.elapsed_seconds().sin() * Vec3::X + cube.starting_position;
    }
}
