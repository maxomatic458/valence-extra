mod placement_handler;

use bvh::bvh_resource::BvhResource;
use placement_handler::on_try_place_default;
use std::time::{Duration, Instant};
use valence::{
    ecs::query::QueryData, interact_block::InteractBlockEvent, inventory::HeldItem, prelude::*,
};

/// Attached to every player that is able to build.
#[derive(Component)]
pub struct BuildState {
    /// Last time the player placed a block.
    pub last_place: Instant,
    /// The build config for the player.
    pub build_config: PlayerBuildConfig,
}

impl Default for BuildState {
    fn default() -> Self {
        Self {
            last_place: Instant::now(),
            build_config: PlayerBuildConfig::default(),
        }
    }
}

/// Configuration
pub struct PlayerBuildConfig {
    /// A Cooldown for placing blocks.
    pub place_cooldown: Duration,
    /// A callback when the player tries to place a block.
    /// This function handles the actual placement of blocks.
    ///
    /// The parameters are: `player_entity`, `clicked_pos` (position of the block the player clicked on), `chunk_layer`, `player_inventory`, `held_item`, `direction`.
    /// Returns `true` if the placement was successful.
    pub on_try_place: fn(
        Entity,
        BlockPos,
        &mut ChunkLayer,
        &mut Inventory,
        &HeldItem,
        Direction,
        &BvhResource,
    ) -> bool,
}

impl Default for PlayerBuildConfig {
    fn default() -> Self {
        Self {
            place_cooldown: Duration::ZERO,
            on_try_place: on_try_place_default,
        }
    }
}

pub struct BuildPlugin;

impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, build_system);
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct BuildQuery {
    entity: Entity,
    build_state: &'static mut BuildState,
    inventory: &'static mut Inventory,
    held_item: &'static HeldItem,
}

fn build_system(
    mut clients: Query<BuildQuery>,
    bvh: Res<BvhResource>,
    mut layers: Query<&mut ChunkLayer>,
    mut events: EventReader<InteractBlockEvent>,
) {
    for event in events.read() {
        let Ok(mut build_query) = clients.get_mut(event.client) else {
            continue;
        };

        if build_query.build_state.last_place.elapsed()
            < build_query.build_state.build_config.place_cooldown
        {
            continue;
        }

        if event.hand != Hand::Main {
            continue;
        }

        let mut layer = layers.single_mut();

        if (build_query.build_state.build_config.on_try_place)(
            build_query.entity,
            event.position,
            &mut layer,
            &mut build_query.inventory,
            build_query.held_item,
            event.face,
            &bvh,
        ) {
            build_query.build_state.last_place = Instant::now();
        }
    }
}
