use crate::GameState;
use bevy_asset_loader::prelude::*;
use bevy_godot::prelude::{
    godot_prelude::{AudioStream, AudioStreamPlayer},
    *,
};
#[derive(AssetCollection, Debug)]
pub struct MusicAssets {
    #[asset(path = "art/House In a Forest Loop.ogg.res")]
    bg_music: Handle<GodotResource>,

    #[asset]
    bg_music_player: Handle<ErasedGodotRef>,

    #[asset(path = "art/gameover.wav.res")]
    death_sfx: Handle<GodotResource>,

    #[asset]
    death_sfx_player: Handle<ErasedGodotRef>,
}

pub struct MusicPlugin;
impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::MainMenu).with_system(init_assets))
            .add_system_set(SystemSet::on_enter(GameState::Countdown).with_system(play_bg_music))
            .add_system_set(
                SystemSet::on_enter(GameState::GameOver)
                    .with_system(stop_bg_music)
                    .with_system(play_death_sfx),
            );
    }
}

fn init_assets(
    mut music_assets: ResMut<MusicAssets>,
    mut assets: ResMut<Assets<GodotResource>>,
    mut godot_assets: ResMut<Assets<ErasedGodotRef>>,
    mut scene_tree: SceneTreeRef,
) {
    let bg_music_stream = assets
        .get_mut(&music_assets.bg_music)
        .unwrap()
        .0
        .clone()
        .cast::<AudioStream>()
        .unwrap();

    let death_sfx_stream = assets
        .get_mut(&music_assets.death_sfx)
        .unwrap()
        .0
        .clone()
        .cast::<AudioStream>()
        .unwrap();

    let bg_music_player = AudioStreamPlayer::new().into_shared();
    let death_sfx_player = AudioStreamPlayer::new().into_shared();

    unsafe {
        bg_music_player.assume_safe().set_stream(bg_music_stream);
        death_sfx_player.assume_safe().set_stream(death_sfx_stream);
    }

    unsafe {
        let root = scene_tree.get().root().unwrap().assume_safe();
        root.add_child(bg_music_player, true);
        root.add_child(death_sfx_player, true);
    }

    music_assets.bg_music_player =
        godot_assets.add(unsafe { ErasedGodotRef::new(bg_music_player.assume_unique()) });
    music_assets.death_sfx_player =
        godot_assets.add(unsafe { ErasedGodotRef::new(death_sfx_player.assume_unique()) });
}

fn play_bg_music(music_assets: Res<MusicAssets>, mut godot_assets: ResMut<Assets<ErasedGodotRef>>) {
    godot_assets
        .get_mut(&music_assets.bg_music_player)
        .unwrap()
        .get::<AudioStreamPlayer>()
        .play(0.0);
}

fn stop_bg_music(music_assets: Res<MusicAssets>, mut godot_assets: ResMut<Assets<ErasedGodotRef>>) {
    godot_assets
        .get_mut(&music_assets.bg_music_player)
        .unwrap()
        .get::<AudioStreamPlayer>()
        .stop();
}

fn play_death_sfx(
    music_assets: Res<MusicAssets>,
    mut godot_assets: ResMut<Assets<ErasedGodotRef>>,
) {
    godot_assets
        .get_mut(&music_assets.death_sfx_player)
        .unwrap()
        .get::<AudioStreamPlayer>()
        .play(0.0);
}
