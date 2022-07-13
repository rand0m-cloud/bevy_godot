use bevy::{
    app::*, asset::AssetPlugin, input::InputPlugin, prelude::*, scene::ScenePlugin,
    window::WindowPlugin,
};
use gdnative::prelude::{Basis, Vector3};

pub mod godot_ref;
pub use godot_ref::*;

pub mod transforms;
pub use transforms::*;

pub mod scene_tree;
pub use scene_tree::*;

pub struct GodotCorePlugin;

impl Plugin for GodotCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins_with(DefaultPlugins, |group| {
            group
                .disable::<InputPlugin>()
                .disable::<WindowPlugin>()
                .disable::<AssetPlugin>()
                .disable::<ScenePlugin>();

            #[cfg(feature = "trace")]
            group.disable::<bevy::render::RenderPlugin>();

            group
        })
        .add_plugin(GodotSceneTreePlugin)
        .add_plugin(GodotTransformsPlugin);
    }
}

pub trait IntoBevyTransform {
    fn to_bevy_transform(self) -> bevy::prelude::Transform;
}

impl IntoBevyTransform for gdnative::prelude::Transform {
    fn to_bevy_transform(self) -> bevy::prelude::Transform {
        let quat = self.basis.to_quat();
        let quat = Quat::from_xyzw(quat.x, quat.y, quat.y, quat.w);

        let scale = self.basis.scale();
        let scale = Vec3::new(scale.x, scale.y, scale.z);

        let origin = Vec3::new(self.origin.x, self.origin.y, self.origin.z);

        Transform {
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
