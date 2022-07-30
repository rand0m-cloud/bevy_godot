use crate::AppState;
use bevy_godot::prelude::{bevy_prelude::SystemSet, *};

pub mod enemy;
pub mod score;

pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(score::ScorePlugin)
            .add_plugin(enemy::EnemyPlugin)
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(play_bg_music));
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
