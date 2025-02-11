use std::time::{Duration, Instant};
use valence::{math::Aabb, prelude::*};

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
    /// A Cooldown for placing blocks.
    pub place_cooldown: Duration,
    /// A callback when the player tries to place a block.
    /// If this callback returns `true`, the block will be placed.
    ///
    /// Note: This callback will be called after all other checks e.g. the build cooldown.
    ///
    /// The parameters are: `block_kind`, `position_of_placed_block`, `player_inventory`.
    pub on_try_place: fn(BlockKind, DVec3, &mut Inventory) -> bool,
    /// If placement checks for entity hitboxes should be ignored.
    pub ignore_entity_hitboxes_for_placement: bool,
}

/// Component that is attached to entities that prevent block placement (based on their hitbox).
#[derive(Component, Default)]
pub struct PreventBlockPlacement {
    /// The hitbox to be used for block placement checks.
    ///
    /// If `None` the entity's hitbox will be used.
    pub hitbox: Option<Aabb>,
}

pub struct BuildPlugin;

impl Plugin for BuildPlugin {
    fn build(&self, _app: &mut App) {}
}
