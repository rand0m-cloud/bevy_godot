use crate::GameState;
use bevy_asset_loader::prelude::*;
use bevy_godot::prelude::{
    godot_prelude::{AudioStream, AudioStreamPlayer},
    *,
};
#[derive(AssetCollection, Resource, Debug)]
pub struct MusicAssets {
    #[asset(path = "art/House In a Forest Loop.ogg")]
    bg_music: Handle<GodotResource>,

    #[asset]
    bg_music_player: Handle<ErasedGodotRef>,

    #[asset(path = "art/gameover.wav")]
    death_sfx: Handle<GodotResource>,

    #[asset]
    death_sfx_player: Handle<ErasedGodotRef>,
}

pub struct MusicPlugin;
impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(init_assets.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(play_bg_music.in_schedule(OnEnter(GameState::Countdown)))
            .add_systems((stop_bg_music, play_death_sfx).in_schedule(OnEnter(GameState::GameOver)));
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

    let mut bg_music_player = unsafe { ErasedGodotRef::new(AudioStreamPlayer::new()) };
    let mut death_sfx_player = unsafe { ErasedGodotRef::new(AudioStreamPlayer::new()) };

    bg_music_player
        .get::<AudioStreamPlayer>()
        .set_stream(bg_music_stream);
    death_sfx_player
        .get::<AudioStreamPlayer>()
        .set_stream(death_sfx_stream);

    scene_tree.add_to_root(bg_music_player.get::<Node>());
    scene_tree.add_to_root(death_sfx_player.get::<Node>());

    music_assets.bg_music_player = godot_assets.add(bg_music_player);
    music_assets.death_sfx_player = godot_assets.add(death_sfx_player);
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
