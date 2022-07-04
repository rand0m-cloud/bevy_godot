use crate::prelude::*;

pub struct GodotTransformsPlugin;

impl Plugin for GodotTransformsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, post_update_godot_transforms)
            .add_system_to_stage(CoreStage::PreUpdate, pre_update_godot_transforms)
            .add_system_to_stage(CoreStage::PostUpdate, set_godot_transforms);
    }
}

fn post_update_godot_transforms(
    entities: Query<(&Transform, &ErasedGodotRef), Changed<Transform>>,
    _godot_lock: GodotLock,
) {
    for (transform, reference) in entities.iter() {
        let obj = reference.get::<Spatial>();
        obj.set_transform(transform.to_godot_transform());
    }
}

fn pre_update_godot_transforms(
    mut entities: Query<(&mut Transform, &ErasedGodotRef)>,

    _godot_lock: GodotLock,
) {
    for (mut transform, reference) in entities.iter_mut() {
        let obj = reference.get::<Spatial>();
        *transform = obj.transform().to_bevy_transform();
    }
}

fn set_godot_transforms(
    entities: Query<(&Transform, &ErasedGodotRef), Added<ErasedGodotRef>>,
    _godot_lock: GodotLock,
) {
    for (transform, reference) in entities.iter() {
        reference
            .get::<Spatial>()
            .set_transform(transform.to_godot_transform());
    }
}
