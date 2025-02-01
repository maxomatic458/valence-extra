use std::time::Instant;

use valence::prelude::*;

/// Attached to every player that is able to build.
#[derive(Component)]
pub struct BuildState {
    /// Last time the player placed a block.
    pub last_place: Instant,
    /// The build config for the player.
    pub build_config: PlayerBuildConfig,
}

/// Configuration 
pub struct PlayerBuildConfig {
    
}

pub struct BuildPlugin;

impl Plugin for BuildPlugin {
    fn build(&self, _app: &mut App) {}
}
