use bevy_godot::prelude::*;

pub mod countdown;
pub mod enemy;
pub mod gameover;
pub mod player;
pub mod score;

pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(score::ScorePlugin)
            .add_plugin(enemy::EnemyPlugin)
            .add_plugin(player::PlayerPlugin)
            .add_plugin(gameover::GameoverPlugin)
            .add_plugin(countdown::CountdownPlugin);
    }
}
