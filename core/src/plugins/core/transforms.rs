use crate::prelude::*;

pub struct GodotTransformsPlugin;

impl Plugin for GodotTransformsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(GodotStage::AfterBevy, post_update_godot_transforms)
            .add_system_to_stage(GodotStage::BeforeBevy, pre_update_godot_transforms);
    }
}

fn post_update_godot_transforms(
    _scene_tree: SceneTreeRef,
    mut entities: Query<
        (&Transform, &mut ErasedGodotRef),
        Or<(Added<Transform>, Changed<Transform>)>,
    >,
) {
    for (transform, mut reference) in entities.iter_mut() {
        let obj = reference.get::<Spatial>();
        obj.set_transform(transform.to_godot_transform());
    }
}

fn pre_update_godot_transforms(
    _scene_tree: SceneTreeRef,
    mut entities: Query<(&mut Transform, &mut ErasedGodotRef)>,
) {
    for (mut transform, mut reference) in entities.iter_mut() {
        let godot_transform = reference.get::<Spatial>().transform().to_bevy_transform();
        if *transform != godot_transform {
            *transform = godot_transform;
        }
    }
}
