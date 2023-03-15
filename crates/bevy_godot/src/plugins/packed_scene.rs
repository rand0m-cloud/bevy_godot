use crate::prelude::{
    godot_prelude::{PackedScene, ResourceLoader},
    *,
};
use gdnative::api::packed_scene::GenEditState;

pub struct PackedScenePlugin;

impl Plugin for PackedScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_scene.in_base_set(CoreSet::PostUpdate))
            .register_type::<GodotScene>();
    }
}

/// A to-be-instanced-and-spawned Godot scene.
///
/// [`GodotScene`]s that are spawned/inserted into the bevy world will be instanced from the provided
/// handle/path and the instance will be added as an [`ErasedGodotRef`] in the next PostUpdate stage.
/// (see [`spawn_scene`])
///
/// If [`None`] parent is given, the instanced Godot scene will be added as a child of the current scene.
#[derive(Component, Debug, Clone)]
pub struct GodotScene {
    resource: GodotSceneResource,
    parent: Option<ErasedGodotRef>,
}

#[derive(Debug, Clone)]
enum GodotSceneResource {
    Path(String),
    Handle(Handle<GodotResource>),
}

impl Default for GodotScene {
    fn default() -> Self {
        Self::from_path("", None)
    }
}

impl GodotScene {
    pub fn from_path(path: &str, parent: Option<ErasedGodotRef>) -> Self {
        Self {
            resource: GodotSceneResource::Path(path.to_string()),
            parent,
        }
    }

    pub fn from_handle(handle: &Handle<GodotResource>, parent: Option<ErasedGodotRef>) -> Self {
        Self {
            resource: GodotSceneResource::Handle(handle.clone()),
            parent,
        }
    }
}

#[derive(Component, Debug, Default)]
struct GodotSceneSpawned;

fn spawn_scene(
    mut commands: Commands,
    mut scene_tree: SceneTreeRef,
    mut new_scenes: Query<
        (
            &mut GodotScene,
            Entity,
            Option<&Transform2D>,
            Option<&Transform>,
        ),
        Without<GodotSceneSpawned>,
    >,
    mut assets: ResMut<Assets<GodotResource>>,
) {
    for (mut scene, ent, transform2d, transform) in new_scenes.iter_mut() {
        let resource_loader = ResourceLoader::godot_singleton();
        let packed_scene = match &scene.resource {
            GodotSceneResource::Path(path) => resource_loader
                .load(path, "PackedScene", false)
                .expect("packed scene to load"),
            GodotSceneResource::Handle(handle) => assets
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

        match &mut scene.parent {
            Some(parent) => parent.get::<Node>().add_child(instance, false),
            None => unsafe {
                let scene = scene_tree.get().current_scene().unwrap();
                scene.assume_safe().add_child(instance, false);
            },
        }

        commands
            .entity(ent)
            .insert(unsafe { ErasedGodotRef::new(instance.assume_unique()) })
            .insert(GodotSceneSpawned);
    }
}
