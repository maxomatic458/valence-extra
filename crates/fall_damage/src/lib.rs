use utils::damage::DamageEvent;
use valence::prelude::*;

#[derive(Component, Default)]
pub struct FallingState {
    /// Last position where the entity was on the ground.
    pub fall_start: DVec3,
    pub falling: bool,
    pub falling_state_config: FallingStateConfig,
}

impl FallingState {
    pub fn new(start_pos: DVec3) -> Self {
        Self {
            fall_start: start_pos,
            falling: false,
            falling_state_config: FallingStateConfig::default(),
        }
    }
}

pub struct FallingStateConfig {
    /// The minimum distance the entity can fall without taking damage.
    pub no_damage_distance: f64,
    /// The damage dealt per block (after the no_damage_distance).
    pub damage_per_block: f64,
}

impl Default for FallingStateConfig {
    fn default() -> Self {
        Self {
            no_damage_distance: 3.0,
            damage_per_block: 1.0,
        }
    }
}

impl FallingState {
    pub fn on_ground(&self) -> bool {
        !self.falling
    }
}

pub struct FallDamagePlugin;

impl Plugin for FallDamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fall_damage_system);
    }
}

fn fall_damage_system(
    mut query: Query<(Entity, &mut FallingState, &Position, &Hitbox)>,
    layers: Query<&ChunkLayer, With<EntityLayer>>, // TODO: Get the correct layer that the entity is on
    mut event_writer: EventWriter<DamageEvent>,
) {
    for (entity, mut fall_damage_state, position, hitbox) in query.iter_mut() {
        let layer = layers.single();

        let is_on_ground = utils::is_on_block(&hitbox.get(), layer);

        if is_on_ground {
            if fall_damage_state.falling {
                let blocks_fallen = (fall_damage_state.fall_start.y - position.0.y).max(0.0);

                if blocks_fallen > fall_damage_state.falling_state_config.no_damage_distance {
                    let damage = (blocks_fallen
                        - fall_damage_state.falling_state_config.no_damage_distance)
                        * fall_damage_state.falling_state_config.damage_per_block;

                    if damage > 0.0 {
                        event_writer.send(DamageEvent {
                            victim: entity,
                            attacker: None,
                            damage: damage as f32,
                        });
                    }
                }

                fall_damage_state.falling = false;
                fall_damage_state.fall_start = position.0;
            }
        } else {
            // player is falling
            if fall_damage_state.fall_start.y <= position.0.y {
                fall_damage_state.fall_start.y = position.0.y
            } else {
                fall_damage_state.falling = true;
            }
        }
    }
}
