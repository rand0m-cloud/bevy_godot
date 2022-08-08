use crate::{GameState, Score};
use bevy_godot::prelude::{bevy_prelude::SystemSet, *};

pub struct ScorePlugin;
impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Countdown).with_system(reset_score))
            .add_system(update_score_counter)
            .add_system_set(SystemSet::on_update(GameState::InGame).with_system(give_score))
            .insert_resource(ScoreTimer(Timer::from_seconds(1.0, true)));
    }
}

pub struct ScoreTimer(Timer);

fn reset_score(mut score: ResMut<Score>) {
    score.0 = 0;
}

fn update_score_counter(score: Res<Score>, mut entities: Query<(&Name, &mut ErasedGodotRef)>) {
    if score.is_changed() {
        let mut score_counter_label = entities
            .iter_mut()
            .find_map(|(name, reference)| {
                if name.as_str() == "ScoreLabel" {
                    Some(reference)
                } else {
                    None
                }
            })
            .unwrap();

        score_counter_label
            .get::<Label>()
            .set_text(score.0.to_string());
    }
}

fn give_score(time: Res<Time>, mut timer: ResMut<ScoreTimer>, mut score: ResMut<Score>) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        score.0 += 1;
    }
}
