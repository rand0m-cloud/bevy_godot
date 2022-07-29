use crate::prelude::{bevy_prelude::*, godot_prelude::*, *};
use std::marker::PhantomData;

#[derive(Debug, Component, Default, Copy, Clone)]
pub struct Transform {
    bevy: bevy::prelude::Transform,
    godot: gdnative::prelude::Transform,
}

impl Transform {
    pub fn as_bevy(&self) -> &bevy::prelude::Transform {
        &self.bevy
    }

    pub fn as_bevy_mut(&mut self) -> TransformMutGuard<'_, BevyTransform> {
        self.into()
    }

    pub fn as_godot(&self) -> &gdnative::prelude::Transform {
        &self.godot
    }

    pub fn as_godot_mut(&mut self) -> TransformMutGuard<'_, GodotTransform> {
        self.into()
    }

    fn update_godot(&mut self) {
        self.godot = self.bevy.to_godot_transform();
    }

    fn update_bevy(&mut self) {
        self.bevy = self.godot.to_bevy_transform();
    }
}

impl From<BevyTransform> for Transform {
    fn from(bevy: BevyTransform) -> Self {
        Self {
            bevy,
            godot: bevy.to_godot_transform(),
        }
    }
}

impl From<GodotTransform> for Transform {
    fn from(godot: GodotTransform) -> Self {
        Self {
            bevy: godot.to_bevy_transform(),
            godot,
        }
    }
}

#[derive(Copy, Clone)]
enum TransformRequested {
    Bevy,
    Godot,
}

pub struct TransformMutGuard<'a, T>(&'a mut Transform, TransformRequested, PhantomData<T>);

impl<'a> std::ops::Deref for TransformMutGuard<'a, GodotTransform> {
    type Target = GodotTransform;
    fn deref(&self) -> &Self::Target {
        &self.0.godot
    }
}

impl<'a> std::ops::DerefMut for TransformMutGuard<'a, GodotTransform> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.godot
    }
}

impl<'a> std::ops::Deref for TransformMutGuard<'a, BevyTransform> {
    type Target = BevyTransform;
    fn deref(&self) -> &Self::Target {
        &self.0.bevy
    }
}

impl<'a> std::ops::DerefMut for TransformMutGuard<'a, BevyTransform> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.bevy
    }
}

impl<'a> From<&'a mut Transform> for TransformMutGuard<'a, GodotTransform> {
    fn from(transform: &'a mut Transform) -> Self {
        TransformMutGuard(transform, TransformRequested::Godot, PhantomData)
    }
}

impl<'a> From<&'a mut Transform> for TransformMutGuard<'a, BevyTransform> {
    fn from(transform: &'a mut Transform) -> Self {
        TransformMutGuard(transform, TransformRequested::Bevy, PhantomData)
    }
}

impl<'a, T> Drop for TransformMutGuard<'a, T> {
    fn drop(&mut self) {
        match self.1 {
            TransformRequested::Bevy => self.0.update_godot(),
            TransformRequested::Godot => self.0.update_bevy(),
        }
    }
}

pub trait IntoBevyTransform {
    fn to_bevy_transform(self) -> bevy::prelude::Transform;
}

impl IntoBevyTransform for gdnative::prelude::Transform {
    fn to_bevy_transform(self) -> bevy::prelude::Transform {
        use bevy::prelude::Quat;

        let quat = self.basis.to_quat();
        let quat = Quat::from_xyzw(quat.x, quat.y, quat.z, quat.w);

        let scale = self.basis.scale();
        let scale = Vec3::new(scale.x, scale.y, scale.z);

        let origin = Vec3::new(self.origin.x, self.origin.y, self.origin.z);

        bevy::prelude::Transform {
            rotation: quat,
            translation: origin,
            scale,
        }
    }
}

pub trait IntoGodotTransform {
    fn to_godot_transform(self) -> gdnative::prelude::Transform;
}

impl IntoGodotTransform for bevy::prelude::Transform {
    fn to_godot_transform(self) -> gdnative::prelude::Transform {
        use gdnative::prelude::Quat;

        let [x, y, z, w] = self.rotation.to_array();
        let quat = Quat::new(x, y, z, w);

        let [x, y, z] = self.scale.to_array();
        let scale = Vector3::new(x, y, z);

        let basis = Basis::from_quat(quat).scaled(scale);

        let [x, y, z] = self.translation.to_array();
        gdnative::prelude::Transform {
            basis,
            origin: Vector3::new(x, y, z),
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]

pub struct Transform2D(pub gdnative::prelude::Transform2D);

impl From<gdnative::prelude::Transform2D> for Transform2D {
    fn from(transform: gdnative::prelude::Transform2D) -> Self {
        Self(transform)
    }
}

impl std::ops::Deref for Transform2D {
    type Target = gdnative::prelude::Transform2D;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Transform2D {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct GodotTransformsPlugin;

impl Plugin for GodotTransformsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, post_update_godot_transforms)
            .add_system_to_stage(CoreStage::PreUpdate, pre_update_godot_transforms)
            .add_system_to_stage(CoreStage::PostUpdate, post_update_godot_transforms_2d)
            .add_system_to_stage(CoreStage::PreUpdate, pre_update_godot_transforms_2d);
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
        if obj.transform() != *transform.as_godot() {
            obj.set_transform(*transform.as_godot());
        }
    }
}

fn pre_update_godot_transforms(
    _scene_tree: SceneTreeRef,
    mut entities: Query<(&mut Transform, &mut ErasedGodotRef)>,
) {
    for (mut transform, mut reference) in entities.iter_mut() {
        let godot_transform = reference.get::<Spatial>().transform();
        if *transform.as_godot() != godot_transform {
            *transform.as_godot_mut() = godot_transform;
        }
    }
}

fn post_update_godot_transforms_2d(
    _scene_tree: SceneTreeRef,
    mut entities: Query<
        (&Transform2D, &mut ErasedGodotRef),
        Or<(Added<Transform2D>, Changed<Transform2D>)>,
    >,
) {
    for (transform, mut reference) in entities.iter_mut() {
        let obj = reference.get::<Node2D>();
        let obj_transform = GodotTransform2D::from_rotation_translation_scale(
            obj.position(),
            obj.rotation() as f32,
            obj.scale(),
        );

        if obj_transform != **transform {
            obj.set_transform(**transform);
        }
    }
}

fn pre_update_godot_transforms_2d(
    _scene_tree: SceneTreeRef,
    mut entities: Query<(&mut Transform2D, &mut ErasedGodotRef)>,
) {
    for (mut transform, mut reference) in entities.iter_mut() {
        let obj = reference.get::<Node2D>();
        let obj_transform = GodotTransform2D::from_rotation_translation_scale(
            obj.position(),
            obj.rotation() as f32,
            obj.scale(),
        );

        if obj_transform != **transform {
            **transform = obj_transform;
        }
    }
}
