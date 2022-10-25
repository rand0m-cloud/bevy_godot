use crate::prelude::{
    godot_prelude::{PackedScene, ResourceLoader},
    *,
};
use gdnative::api::packed_scene::GenEditState;

pub struct PackedScenePlugin;

impl Plugin for PackedScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, spawn_scene)
            .register_type::<GodotScene>();
    }
}

#[derive(Component, Debug, Reflect, Clone)]
#[reflect(Component)]
pub enum GodotScene {
    ResourcePath(String),
    ResourceHandle(Handle<GodotResource>),
}

impl Default for GodotScene {
    fn default() -> Self {
        Self::from_path("")
    }
}

impl GodotScene {
    pub fn from_path(path: &str) -> Self {
        Self::ResourcePath(path.to_string())
    }

    pub fn from_handle(handle: &Handle<GodotResource>) -> Self {
        Self::ResourceHandle(handle.clone())
    }
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
struct GodotSceneSpawned;

fn spawn_scene(
    mut commands: Commands,
    mut scene_tree: SceneTreeRef,
    new_scenes: Query<
        (
            &GodotScene,
            Entity,
            Option<&Transform2D>,
            Option<&Transform>,
        ),
        Without<GodotSceneSpawned>,
    >,
    mut assets: ResMut<Assets<GodotResource>>,
) {
    for (scene, ent, transform2d, transform) in new_scenes.iter() {
        let resource_loader = ResourceLoader::godot_singleton();
        let packed_scene = match scene {
            GodotScene::ResourcePath(path) => resource_loader
                .load(path, "PackedScene", false)
                .expect("packed scene to load"),
            GodotScene::ResourceHandle(handle) => assets
                .get_mut(handle)
                .expect("packed scene to exist in assets")
                .0
                .clone(),
        };

        let instance = unsafe {
            packed_scene
                .cast::<PackedScene>()
                .expect("resource to be a packed scene")
                .assume_safe()
                .instance(GenEditState::DISABLED.0)
                .unwrap()
        };

        if let Some(transform2d) = transform2d {
            unsafe {
                instance
                    .assume_safe()
                    .cast::<Node2D>()
                    .unwrap()
                    .set_transform(**transform2d);
            }
        }

        if let Some(transform) = transform {
            unsafe {
                instance
                    .assume_safe()
                    .cast::<Spatial>()
                    .unwrap()
                    .set_transform(*transform.as_godot());
            }
        }

        unsafe {
            let scene = scene_tree.get().current_scene().unwrap();
            scene.assume_safe().add_child(instance, false);
        }

        commands
            .entity(ent)
            .insert(unsafe { ErasedGodotRef::new(instance.assume_unique()) })
            .insert(GodotSceneSpawned);
    }
}
