pub mod utils;

use ::utils::aaab::AabbExt;
use bevy_ecs::query::QueryData;
use bevy_time::Time;
use bvh::bvh_resource::{BvhResource, EntityBvhEntry, ENTITY_BLOCK_BVH_IDX, ENTITY_ENTITY_BVH_IDX};
use utils::swept_aabb_collide;
use valence::{entity::Velocity, math::Aabb, prelude::*};

/// The acceleration of an entity.
#[derive(Component)]
pub struct Acceleration(pub Vec3);

/// The maximum speed that an entity can have.
#[derive(Component)]
pub struct SpeedLimit(pub f32);

/// The drag that will multiply the entity's velocity by the drag every second.
#[derive(Component)]
pub struct Drag(pub Vec3);

// TODO: add this for entity collisions as well
// + make this configurable per movement axis.

/// Sets the entity's velocity to zero if the entity collides with a block on the given face.
///
/// If you want to stop an entity when it touches the top of the block, the face should be `Direction::Up`.
#[derive(Component)]
pub struct StopOnBlockCollision(u8);

impl StopOnBlockCollision {
    pub fn new(faces: Vec<Direction>) -> Self {
        let mut face = 0;
        for direction in faces {
            face |= 1 << direction as u8;
        }
        Self(face)
    }

    /// Stops the entity when it collides with the ground (the top face of a block).
    pub fn ground() -> Self {
        Self::new(vec![Direction::Up])
    }

    /// Stops the entity when it collides with any face of a block.
    pub fn all() -> Self {
        Self(u8::MAX)
    }

    /// If the entity should stop when it collides with the given face.
    pub fn should_stop(&self, face: Direction) -> bool {
        self.0 & (1 << face as u8) != 0
    }

    /// If the entity should stop given a block face bitmap.
    pub fn should_stop_bitmap(&self, bitmap: u8) -> bool {
        self.0 & bitmap != 0
    }
}

/// The config for entity-entity collisions.
#[derive(Component, Default)]
pub struct EntityCollisionConfig {
    /// The hitbox that will be used for entity collision detection.
    ///
    /// If `None`, the entity's hitbox will be used.
    pub entity_collider_hitbox: Option<Aabb>,
}

/// The config for entity-block collisions.
#[derive(Component, Default)]
pub struct BlockCollisionConfig {
    /// The hitbox that will be used for block collision detection.
    ///
    /// If `None`, the entity's hitbox will be used.
    // TODO: have the option to register collisions without stopping the entity
    // from going to the block.
    pub block_collider_hitbox: Option<Aabb>,
}

/// The event emitted when an entity collides with another entity.
#[derive(Event, Debug)]
pub struct EntityEntityCollisionEvent {
    /// This entity is the one that performed the collision detection.
    pub entity1: Entity,
    pub entity2: Entity,
}

/// The event emitted when an entity collides with a block.
#[derive(Event, Debug)]
pub struct EntityBlockCollisionEvent {
    pub entity: Entity,
    pub block_pos: BlockPos,
    /// Bitmap of the block faces that the entity collided with.
    pub block_face_bitmap: u8,
}

impl EntityBlockCollisionEvent {}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EntityEntityCollisionEvent>()
            .add_event::<EntityBlockCollisionEvent>()
            .insert_resource(BvhResource::with_bvhs(2))
            .add_systems(PreUpdate, (physics_system, rebuild_bvh));
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct PhysicsQuery {
    pub entity: Entity,
    pub position: &'static mut Position,
    pub velocity: &'static mut Velocity,
    pub acceleration: Option<&'static Acceleration>,
    pub hitbox: &'static Hitbox,
    pub drag: Option<&'static Drag>,
    pub speed_limit: Option<&'static SpeedLimit>,
    pub stop_on_block_collision: Option<&'static StopOnBlockCollision>,
    pub entity_collision_config: Option<&'static EntityCollisionConfig>,
    pub block_collision_config: Option<&'static BlockCollisionConfig>,
}

fn physics_system(
    bvh: ResMut<BvhResource>,
    time: Res<Time>,
    mut query: Query<PhysicsQuery, Without<Client>>,
    mut entity_entity_collision_writer: EventWriter<EntityEntityCollisionEvent>,
    mut entity_block_collision_writer: EventWriter<EntityBlockCollisionEvent>,
    // TODO: support for multiple layers
    layer: Query<&ChunkLayer, With<EntityLayer>>,
) {
    /// Helper function to help with creating the ranges used for aabb broadphase.
    fn create_range(
        start: i32,
        step: i32,
        steps: i32,
        center: i32,
    ) -> std::ops::RangeInclusive<i32> {
        if step > 0 {
            start - step * (steps + 1)..=center + step * (steps + 2)
        } else {
            center + step * (steps + 2)..=start - step * (steps + 1)
        }
    }

    enum PhysicsEvent {
        EntityEntityCollision(EntityEntityCollisionEvent),
        EntityBlockCollision(EntityBlockCollisionEvent),
    }

    let (tx, rx) = std::sync::mpsc::channel::<PhysicsEvent>();

    query.iter_mut().for_each(|mut entity| {
        if let Some(drag) = entity.drag {
            entity.velocity.0 *= 1.0 - drag.0 * time.delta_seconds();
        }

        if let Some(acceleration) = entity.acceleration {
            entity.velocity.0 += acceleration.0 * time.delta_seconds();
        }

        if let Some(speed_limit) = entity.speed_limit {
            entity.velocity.0 = entity.velocity.0.clamp_length(0.0, speed_limit.0);
        }

        // TODO: support for multiple layers
        let layer = layer.single();

        let _old_velocity = entity.velocity.0;

        if let Some(block_collision_config) = entity.block_collision_config {
            let entity_hitbox = block_collision_config
                .block_collider_hitbox
                .unwrap_or(entity.hitbox.get());

            for _ in 0..3 {
                let velocity_delta = entity.velocity.0 * time.delta_seconds();
                let (vx, vy, vz) = (velocity_delta.x, velocity_delta.y, velocity_delta.z);

                let (step_x, step_y, step_z) = (
                    if vx > 0.0 { 1 } else { -1 },
                    if vy > 0.0 { 1 } else { -1 },
                    if vz > 0.0 { 1 } else { -1 },
                );

                let (steps_x, steps_y, steps_z) = (
                    (entity_hitbox.width_x() / 2.0) as i32,
                    (entity_hitbox.width_y() / 2.0) as i32,
                    (entity_hitbox.width_z() / 2.0) as i32,
                );

                let (x, y, z) = (
                    entity.position.0.x as i32,
                    entity.position.0.y as i32,
                    entity.position.0.z as i32,
                );

                let (cx, cy, cz) = (
                    (entity.position.0.x + velocity_delta.x as f64) as i32,
                    (entity.position.0.y + velocity_delta.y as f64) as i32,
                    (entity.position.0.z + velocity_delta.z as f64) as i32,
                );

                let mut potential_collisions = Vec::new();

                let x_range = create_range(x, step_x, steps_x, cx);
                for i in x_range.step_by(step_x.unsigned_abs() as usize) {
                    let y_range = create_range(y, step_y, steps_y, cy);

                    for j in y_range.step_by(step_y.unsigned_abs() as usize) {
                        let z_range = create_range(z, step_z, steps_z, cz);

                        for k in z_range.step_by(step_z.unsigned_abs() as usize) {
                            let block_pos = BlockPos { x: i, y: j, z: k };
                            let block = layer.block(block_pos);

                            let Some(block) = block else {
                                continue;
                            };

                            if block.state.is_air() {
                                continue;
                            }

                            for collider in block.state.collision_shapes() {
                                let block_aabb =
                                    collider.translate(DVec3::new(i as f64, j as f64, k as f64));

                                let Some(collision) = swept_aabb_collide(
                                    &entity_hitbox,
                                    &velocity_delta,
                                    &block_aabb,
                                ) else {
                                    continue;
                                };

                                if collision.face_direction.x.is_none()
                                    && collision.face_direction.y.is_none()
                                    && collision.face_direction.z.is_none()
                                {
                                    continue;
                                }

                                potential_collisions.push((block_pos, collision));
                            }
                        }
                    }
                }

                if potential_collisions.is_empty() {
                    break;
                }

                let (block_pos, mut collision) = potential_collisions
                    .into_iter()
                    .min_by(|a, b| a.1.entry_time.partial_cmp(&b.1.entry_time).unwrap())
                    .unwrap();

                collision.entry_time -= 0.01;

                let mut collision_bitmap = 0;

                if let Some(normal_x) = collision.face_direction.x {
                    entity.velocity.0.x = 0.0;
                    entity.position.0.x += vx as f64 * collision.entry_time;
                    let direction = if normal_x {
                        Direction::East
                    } else {
                        Direction::West
                    };
                    collision_bitmap |= 1 << direction as u8;
                }

                if let Some(normal_y) = collision.face_direction.y {
                    entity.velocity.0.y = 0.0;
                    entity.position.0.y += vy as f64 * collision.entry_time;
                    let direction = if normal_y {
                        Direction::Up
                    } else {
                        Direction::Down
                    };
                    collision_bitmap |= 1 << direction as u8;
                }

                if let Some(normal_z) = collision.face_direction.z {
                    entity.velocity.0.z = 0.0;
                    entity.position.0.z += vz as f64 * collision.entry_time;
                    let direction = if normal_z {
                        Direction::South
                    } else {
                        Direction::North
                    };
                    collision_bitmap |= 1 << direction as u8;
                }

                let event = EntityBlockCollisionEvent {
                    entity: entity.entity,
                    block_pos,
                    block_face_bitmap: collision_bitmap,
                };

                if let Some(stop_on_block_collision) = entity.stop_on_block_collision {
                    if stop_on_block_collision.should_stop_bitmap(collision_bitmap) {
                        entity.velocity.0 = Vec3::ZERO;
                    }
                }

                tx.send(PhysicsEvent::EntityBlockCollision(event)).unwrap();
            }
        }

        entity.position.0 += (entity.velocity.0 * time.delta_seconds()).as_dvec3();

        // TODO: entity collision

        if let Some(entity_collision_config) = entity.entity_collision_config {
            let aabb = entity_collision_config
                .entity_collider_hitbox
                .unwrap_or(entity.hitbox.get());

            for other in bvh[ENTITY_ENTITY_BVH_IDX].get_in_range(aabb) {
                if other.entity == entity.entity {
                    continue;
                }

                entity_entity_collision_writer.send(EntityEntityCollisionEvent {
                    entity1: entity.entity,
                    entity2: other.entity,
                });
            }
        }
    });

    for event in rx.try_iter() {
        match event {
            PhysicsEvent::EntityEntityCollision(event) => {
                entity_entity_collision_writer.send(event);
            }
            PhysicsEvent::EntityBlockCollision(event) => {
                entity_block_collision_writer.send(event);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn rebuild_bvh(
    query: Query<PhysicsQuery, Or<(With<EntityCollisionConfig>, With<BlockCollisionConfig>)>>,
    mut bvh: ResMut<BvhResource>,
) {
    if query.is_empty() {
        return;
    }

    let mut entity_entity_colls = vec![];
    let mut entity_block_colls = vec![];

    for entity in query.iter() {
        if let Some(entity_collision_config) = entity.entity_collision_config {
            let aabb = match entity_collision_config.entity_collider_hitbox {
                Some(hitbox) => hitbox.translate(entity.position.0),
                None => entity.hitbox.get(),
            };

            entity_entity_colls.push(EntityBvhEntry {
                entity: entity.entity,
                hitbox: aabb,
            });
        }

        if let Some(block_collision_config) = entity.block_collision_config {
            let aabb = match block_collision_config.block_collider_hitbox {
                Some(hitbox) => hitbox.translate(entity.position.0),
                None => entity.hitbox.get(),
            };

            entity_block_colls.push(EntityBvhEntry {
                entity: entity.entity,
                hitbox: aabb,
            });
        }
    }

    bvh.get_mut(ENTITY_ENTITY_BVH_IDX)
        .unwrap()
        .build(entity_entity_colls);
    bvh.get_mut(ENTITY_BLOCK_BVH_IDX)
        .unwrap()
        .build(entity_block_colls);
}
