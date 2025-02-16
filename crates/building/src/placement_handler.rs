use bvh::bvh_resource::{BvhResource, ENTITY_BLOCK_BVH_IDX};
use valence::{
    block::{BlockKind, PropName, PropValue},
    inventory::HeldItem,
    math::{Aabb, DVec3},
    prelude::{Entity, Inventory},
    BlockPos, BlockState, ChunkLayer, Direction, ItemStack,
};

/// A default implementation for the block placement handler.
/// That mimics vanilla Minecraft behavior.
pub fn on_try_place_default(
    _player_entity: Entity,
    clicked_pos: BlockPos,
    chunk_layer: &mut ChunkLayer,
    player_inventory: &mut Inventory,
    held_item: &HeldItem,
    direction: Direction,
    bvh: &BvhResource,
) -> bool {
    let slot_id = held_item.slot();
    let stack = player_inventory.slot(slot_id);

    if stack.count == 0 {
        // The player is not holding any items.
        return false;
    }

    let Some(block_kind) = BlockKind::from_item_kind(stack.item) else {
        // The item the player is holding cannot be placed as a block.
        return false;
    };

    let block_state = BlockState::from_kind(block_kind);
    let block_hitboxes = block_state.collision_shapes();

    let real_pos = clicked_pos.get_in_direction(direction);

    for mut block_hitbox in block_hitboxes {
        let tolerance = DVec3::new(0.0, 0.01, 0.0);
        block_hitbox = Aabb::new(
            block_hitbox.min()
                + DVec3::new(real_pos.x as f64, real_pos.y as f64, real_pos.z as f64)
                + tolerance,
            block_hitbox.max()
                + DVec3::new(real_pos.x as f64, real_pos.y as f64, real_pos.z as f64)
                - tolerance,
        );

        if bvh[ENTITY_BLOCK_BVH_IDX]
            .get_in_range(block_hitbox)
            .next()
            .is_some()
        {
            // TODO: this ignores the `BlockCollisionConfig` as defined in physics.
            // The block would intersect with another entity.
            return false;
        }
    }

    // The block can be placed.

    if stack.count > 1 {
        let amount = stack.count - 1;
        player_inventory.set_slot_amount(slot_id, amount);
    } else {
        player_inventory.set_slot(slot_id, ItemStack::EMPTY);
    }

    let state = block_kind.to_state().set(
        PropName::Axis,
        match direction {
            Direction::Down | Direction::Up => PropValue::Y,
            Direction::North | Direction::South => PropValue::Z,
            Direction::West | Direction::East => PropValue::X,
        },
    );

    chunk_layer.set_block(real_pos, state);

    true
}
