use crate::AppState;
use bevy_godot::prelude::{bevy_prelude::SystemSet, *};

pub struct MusicPlugin;
impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Countdown).with_system(play_bg_music))
            .add_system_set(
                SystemSet::on_enter(AppState::GameOver)
                    .with_system(stop_bg_music)
                    .with_system(play_death_sfx),
            );
    }
}

fn play_bg_music(mut entities: Query<(&Name, &mut ErasedGodotRef)>) {
    let mut music = entities
        .iter_mut()
        .find_map(|(name, reference)| {
            if name.as_str() == "Music" {
                Some(reference)
            } else {
                None
            }
        })
        .unwrap();

    music.get::<AudioStreamPlayer>().play(0.0);
}

fn stop_bg_music(mut entities: Query<(&Name, &mut ErasedGodotRef)>) {
    let mut music = entities
        .iter_mut()
        .find_map(|(name, reference)| {
            if name.as_str() == "Music" {
                Some(reference)
            } else {
                None
            }
        })
        .unwrap();

    music.get::<AudioStreamPlayer>().stop();
}

fn play_death_sfx(mut entities: Query<(&Name, &mut ErasedGodotRef)>) {
    let mut death_sfx = entities
        .iter_mut()
        .find_map(|(name, reference)| {
            if name.as_str() == "DeathSound" {
                Some(reference)
            } else {
                None
            }
        })
        .unwrap();

    death_sfx.get::<AudioStreamPlayer>().play(0.0);
}
