use crate::Bvh;
use valence::{math::Aabb, prelude::*};

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
    pub bvh: Bvh<EntityBvhEntry>,
}

impl BvhResource {
    /// Clear the BVH.
    pub fn clear(&mut self) {
        self.bvh.clear();
    }

    /// Build the BVH from the given entries.
    pub fn build(&mut self, entries: Vec<EntityBvhEntry>) {
        self.bvh = Bvh::build(entries, |entry| entry.hitbox);
    }

    /// Get all entities that are contained or intersect with the given AABB.
    pub fn get_in_range(&self, target: Aabb) -> impl Iterator<Item = &EntityBvhEntry> + '_ {
        self.bvh.range(target, move |entry| entry.hitbox)
    }
}
