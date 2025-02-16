use std::collections::HashMap;
use valence::{math::Aabb, prelude::*};

/// Use the BVH with key `0` for entity-entity collisions.
pub const ENTITY_ENTITY_BVH_IDX: u64 = 0;
/// Use the BVH with key `1` for entity-block collisions.
pub const ENTITY_BLOCK_BVH_IDX: u64 = 1;

/// Represents an entity that is stored within
#[derive(Debug, Clone, Copy)]
pub struct EntityBvhEntry {
    /// The bevy entity.
    pub entity: Entity,
    /// The hitbox used for collision detection.
    pub hitbox: Aabb,
}

// TODO: make this not a resource so it can be per layer

/// A resource that stores a BVH for all entities that are able to collide
/// with each other.
#[derive(Resource, Default)]
pub struct BvhResource {
    /// A Vec of BVHs, that can be used for different kinds of hitboxes.
    bvhs: HashMap<u64, Bvh>,
}

impl std::ops::Index<u64> for BvhResource {
    type Output = Bvh;

    fn index(&self, index: u64) -> &Self::Output {
        self.bvhs.get(&index).unwrap()
    }
}

impl BvhResource {
    pub fn get_mut(&mut self, index: u64) -> Option<&mut Bvh> {
        self.bvhs.get_mut(&index)
    }

    pub fn with_bvhs(num: usize) -> Self {
        let mut bvhs = HashMap::with_capacity(num);
        for i in 0..num {
            bvhs.insert(i as u64, Bvh(crate::Bvh::default()));
        }

        Self { bvhs }
    }
}

/// A BVH for entities that are able to collide with each other.
pub struct Bvh(crate::Bvh<EntityBvhEntry>);

impl Bvh {
    /// Clear the BVH.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Build the BVH from the given entries.
    pub fn build(&mut self, entries: Vec<EntityBvhEntry>) {
        self.0 = crate::Bvh::build(entries, |entry| entry.hitbox);
    }

    /// Get all entities that are contained or intersect with the given AABB.
    pub fn get_in_range(&self, target: Aabb) -> impl Iterator<Item = &EntityBvhEntry> + '_ {
        self.0.range(target, move |entry| entry.hitbox)
    }
}
