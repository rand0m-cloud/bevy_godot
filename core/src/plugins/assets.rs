use crate::prelude::{
    godot_prelude::{GodotError, RefCounted, SubClass, ThreadLocal},
    *,
};
use bevy::asset::{AssetLoader, AssetPlugin, AssetServerSettings, LoadContext, LoadedAsset};
use bevy::reflect::TypeUuid;
use bevy::utils::BoxedFuture;
use std::time::Duration;

pub struct GodotAssetsPlugin;
impl Plugin for GodotAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AssetServerSettings {
            asset_folder: std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            watch_for_changes: false,
        })
        .add_plugin(AssetPlugin)
        .add_asset::<GodotResource>()
        .add_asset::<ErasedGodotRef>()
        .init_asset_loader::<GodotResourceLoader>();
    }
}

#[derive(Default)]
pub struct GodotResourceLoader;

#[derive(Debug, TypeUuid)]
#[uuid = "c3bd07de-eade-4cb0-9392-7c21394286f8"]
pub struct GodotResource(pub Ref<Resource>);

impl GodotResource {
    pub fn get<T: GodotObject<Memory = RefCounted> + SubClass<Resource>>(
        &mut self,
    ) -> Option<Ref<T, ThreadLocal>> {
        unsafe { self.0.clone().assume_thread_local().cast() }
    }
}

impl AssetLoader for GodotResourceLoader {
    fn load<'a>(
        &'a self,
        _bytes: &'a [u8],
        load_context: &'a mut LoadContext<'_>,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async {
            let mut load_asset = || {
                let resource_loader = ResourceLoader::godot_singleton();
                let loader = resource_loader
                    .load_interactive(
                        "res://".to_owned()
                            + load_context.path().to_str().ok_or_else(|| {
                                anyhow::anyhow!("failed to convert asset path to string")
                            })?,
                        "",
                    )
                    .ok_or_else(|| {
                        anyhow::anyhow!("failed to load asset {}", load_context.path().display())
                    })?;

                loop {
                    match unsafe { loader.assume_safe().poll() } {
                        Ok(()) => {}
                        Err(GodotError::FileEof) => break,
                        Err(e) => return Err(anyhow::anyhow!("failed to load godot asset: {}", e)),
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                unsafe {
                    let resource = loader.assume_safe().get_resource().unwrap();
                    load_context.set_default_asset(LoadedAsset::new(GodotResource(resource)));
                }

                Ok(())
            };

            if let Err(e) = load_asset() {
                eprintln!(
                    "loading {} asset failed: {}",
                    load_context.path().to_str().unwrap(),
                    e
                );
                return Err(e);
            }

            Ok(())
        })
    }
    fn extensions(&self) -> &[&str] {
        &["tscn", "scn", "res", "tres"]
    }
}
