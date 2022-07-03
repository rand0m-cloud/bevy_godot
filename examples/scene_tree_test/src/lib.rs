#![allow(clippy::type_complexity)]

use bevy::core::Timer;
use bevy_godot::prelude::*;
use gdnative::api::CSGBox;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_system(spawn_cube)
        .add_system(cube_lifetime)
        .add_system(print_entities)
        .insert_resource(CubeTimer(Timer::from_seconds(0.2, true)));
}

bevy_godot_init!(init, build_app);

#[derive(Component)]
pub struct Cube {
    pub lifetime: Timer,
}
pub struct CubeTimer(pub Timer);

fn spawn_cube(
    mut commands: Commands,
    scene_tree: Res<SceneTreeRef>,
    _godot_lock: GodotLock,
    mut timer: ResMut<CubeTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        let csg_node = CSGBox::new().into_shared();
        unsafe {
            scene_tree
                .get()
                .current_scene()
                .unwrap()
                .assume_safe()
                .add_child(csg_node, false);
        }

        commands
            .spawn()
            .insert(Cube {
                lifetime: Timer::from_seconds(3.0, false),
            })
            .insert(ErasedGodotRef::new(unsafe { csg_node.assume_unique() }))
            .insert(Transform::from_translation(Vec3::new(
                10.0 * f64::sin(time.seconds_since_startup()) as f32,
                5.0 * f32::sin(time.seconds_since_startup() as f32),
                -8.0 + -1.0 * time.seconds_since_startup() as f32,
            )))
            .insert(Children::default());
    }
}

fn cube_lifetime(
    mut commands: Commands,
    mut cubes: Query<(&mut Cube, &ErasedGodotRef, Entity)>,
    time: Res<Time>,
    _godot_lock: GodotLock,
) {
    for (mut cube, reference, ent) in cubes.iter_mut() {
        cube.lifetime.tick(time.delta());
        if cube.lifetime.finished() {
            reference.get::<Node>().queue_free();
            commands.entity(ent).despawn_recursive();
        }
    }
}

fn print_entities(
    entities: Query<(
        Entity,
        Option<&Name>,
        Option<&ErasedGodotRef>,
        Option<&Parent>,
    )>,
    _godot_lock: GodotLock,
) {
    for (ent, name, reference, parent) in entities.iter() {
        let instance_id = reference.map(|r| r.get::<Object>().get_instance_id());

        println!(
            "{} [B: {:?}, G: {}, Parent: {:?}]",
            name.unwrap_or(&Name::new("N/A")),
            ent,
            instance_id.unwrap_or(-1),
            parent
        );
    }
}
