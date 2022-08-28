use crate::prelude::{
    bevy_prelude::*,
    godot_prelude::{GodotError, RefCounted, SubClass, ThreadLocal},
    *,
};
use bevy::asset::*;
use bevy::reflect::TypeUuid;
use bevy::utils::BoxedFuture;
use std::{
    future::Future,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    pin::Pin,
    time::Duration,
};

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
        .insert_resource(AssetServer::with_boxed_io(Box::new(GodotAssetIo)))
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

fn make_godot_path(path: &Path) -> String {
    format!("res://{}", path.to_str().expect("path to be valid unicode"))
}

trait GodotErrorExt<T> {
    fn map_godot_error(self) -> Result<T, AssetIoError>;
}

impl<T> GodotErrorExt<T> for Result<T, GodotError> {
    fn map_godot_error(self) -> Result<T, AssetIoError> {
        self.map_err(|e| AssetIoError::Io(Error::new(ErrorKind::Other, e.to_string())))
    }
}

pub struct GodotAssetIo;
impl AssetIo for GodotAssetIo {
    fn load_path<'a>(
        &'a self,
        path: &'a Path,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, AssetIoError>> + Send + 'a>> {
        Box::pin(async {
            let godot_path = make_godot_path(path);

            let resource_loader = ResourceLoader::godot_singleton();
            if !resource_loader.exists(godot_path, "") {
                return Err(AssetIoError::NotFound(path.to_path_buf()));
            }

            Ok(vec![])
        })
    }
    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf> + 'static>, AssetIoError> {
        let godot_path = make_godot_path(path);

        let mut entries = vec![];

        let dir = Directory::new();
        dir.open(godot_path).map_godot_error()?;
        dir.list_dir_begin(true, false).map_godot_error()?;

        let mut file;

        loop {
            file = dir.get_next();
            let file = file.to_string();
            if file == "" {
                break;
            }
            entries.push(PathBuf::from(file));
        }

        Ok(Box::new(entries.into_iter()))
    }
    fn get_metadata(&self, path: &Path) -> Result<Metadata, AssetIoError> {
        let godot_path = make_godot_path(path);

        let dir = Directory::new();
        let entry_type = if dir.dir_exists(&godot_path) {
            FileType::Directory
        } else if dir.file_exists(&godot_path) {
            FileType::File
        } else {
            return Err(AssetIoError::Io(Error::new(
                ErrorKind::Other,
                "path is not a directory or a file",
            )));
        };

        Ok(Metadata::new(entry_type))
    }
    fn watch_path_for_changes(&self, _path: &Path) -> Result<(), AssetIoError> {
        Ok(())
    }
    fn watch_for_changes(&self) -> Result<(), AssetIoError> {
        Ok(())
    }

    fn is_dir(&self, path: &Path) -> bool {
        let godot_path = make_godot_path(path);
        let dir = Directory::new();
        dir.dir_exists(godot_path)
    }
    fn is_file(&self, path: &Path) -> bool {
        let godot_path = make_godot_path(path);
        let dir = Directory::new();
        dir.file_exists(godot_path)
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
