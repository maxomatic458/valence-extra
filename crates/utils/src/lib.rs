pub mod damage;
use valence::{math::Aabb, prelude::*};

/// Returns a list of all the blocks that are inside (or intersect) the given AABB
pub fn aabb_full_block_intersections(aabb: &Aabb) -> Vec<BlockPos> {
    let mut blocks = Vec::new();

    let min = aabb.min().floor();
    let max = aabb.max().ceil();

    for x in (min.x as i32)..(max.x as i32) {
        for y in (min.y as i32)..(max.y as i32) {
            for z in (min.z as i32)..(max.z as i32) {
                blocks.push(BlockPos { x, y, z });
            }
        }
    }

    blocks
}

/// Returns true if the AABB is on a block
pub fn is_on_block(hitbox: &Aabb, layer: &ChunkLayer) -> bool {
    let hitbox = Aabb::new(hitbox.min() + DVec3::new(0.0, -0.001, 0.0), hitbox.max());

    let blocks_below = aabb_full_block_intersections(&hitbox);

    blocks_below.iter().any(|b| {
        if let Some(block) = layer.block(*b) {
            !block.state.is_air()
        } else {
            false
        }
    })
}
