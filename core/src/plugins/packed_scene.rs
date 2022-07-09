use crate::prelude::*;
use gdnative::api::packed_scene::GenEditState;

pub struct PackedScenePlugin;

impl Plugin for PackedScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_scene).register_type::<GodotScene>();
    }
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct GodotScene {
    path: String,
}

impl GodotScene {
    pub fn from_path(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
struct GodotSceneSpawned;

fn spawn_scene(
    mut commands: Commands,
    mut scene_tree: SceneTreeRef,
    new_scenes: Query<(&GodotScene, Entity), Without<GodotSceneSpawned>>,
) {
    for (scene, ent) in new_scenes.iter() {
        let resource_loader = ResourceLoader::godot_singleton();
        let packed_scene = resource_loader
            .load(scene.path.clone(), "PackedScene", false)
            .expect("packed scene to load");

        let instance = unsafe {
            packed_scene
                .cast::<PackedScene>()
                .unwrap()
                .assume_safe()
                .instance(GenEditState::DISABLED.0)
                .unwrap()
        };

        unsafe {
            let scene = scene_tree.get().current_scene().unwrap();
            scene.assume_safe().add_child(instance, false);
        }

        commands
            .entity(ent)
            .insert(unsafe { ErasedGodotRef::new(instance.assume_unique()) })
            .insert(Children::default())
            .insert(GodotSceneSpawned);
    }
}
