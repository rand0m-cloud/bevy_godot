#![allow(clippy::type_complexity)]

use bevy::core::Timer;
use bevy_godot::prelude::*;
use gdnative::api::CSGBox;

fn init(_handle: &InitHandle) {}

fn build_app(app: &mut App) {
    app.add_system(spawn_cube)
        .add_system(cube_lifetime)
        .add_system(print_entities)
        .insert_resource(CubeSpawnTimer(Timer::from_seconds(0.2, true)))
        .insert_resource(PrintEntitiesTimer(Timer::from_seconds(1.0, true)));
}

bevy_godot_init!(init, build_app);

#[derive(Component)]
pub struct Cube {
    pub lifetime: Timer,
}

pub struct CubeSpawnTimer(pub Timer);
pub struct PrintEntitiesTimer(pub Timer);

fn spawn_cube(
    mut commands: Commands,
    mut scene_tree: SceneTreeRef,
    mut timer: ResMut<CubeSpawnTimer>,
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
            .insert(unsafe { ErasedGodotRef::new(csg_node.assume_unique()) })
            .insert(Transform::from(BevyTransform::from_translation(Vec3::new(
                10.0 * time.seconds_since_startup().sin() as f32,
                5.0 * time.seconds_since_startup().sin() as f32,
                -8.0 + -1.0 * time.seconds_since_startup() as f32,
            ))))
            .insert(Children::default());
    }
}

fn cube_lifetime(mut cubes: Query<(&mut Cube, &mut ErasedGodotRef)>, time: Res<Time>) {
    for (mut cube, mut reference) in cubes.iter_mut() {
        cube.lifetime.tick(time.delta());
        if cube.lifetime.finished() {
            reference.get::<Node>().queue_free();
        }
    }
}

fn print_entities(
    mut entities: Query<(
        Entity,
        Option<&Name>,
        Option<&mut ErasedGodotRef>,
        Option<&Children>,
        Option<&Parent>,
    )>,
    time: Res<Time>,
    mut print_timer: ResMut<PrintEntitiesTimer>,
) {
    print_timer.0.tick(time.delta());
    if !print_timer.0.just_finished() {
        return;
    }

    for (ent, name, reference, children, parent) in entities.iter_mut() {
        let instance_id = reference.map(|mut r| r.get::<Object>().get_instance_id());

        println!(
            "{} [B: {:?}, G: {}, Parent: {:?}, Children Count: {}]",
            name.unwrap_or(&Name::new("N/A")),
            ent,
            instance_id.unwrap_or(-1),
            parent,
            children.iter().count()
        );
    }
}
