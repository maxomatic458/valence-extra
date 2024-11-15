use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use bevy_ecs::query::QueryData;
use fall_damage::FallingState;
use valence::{
    entity::{living::StuckArrowCount, EntityId, EntityStatuses, Velocity},
    inventory::HeldItem,
    prelude::*,
};

const _BASE_HIT_COOLDOWN: Duration = Duration::from_millis(500);

#[derive(Resource)]
pub struct CombatConfig {
    pub hit_cooldown: Duration,
}

/// Attached to every player that participates in combat.
#[derive(Component)]
pub struct CombatState {
    /// Last time the player hit another entity.
    pub last_attack: Instant,
    /// The last time the player was hit by another entity.
    pub last_hit: Instant,
    /// The player is sprinting.
    pub sprinting: bool,
    /// The player is sneaking.
    pub sneaking: bool,
    /// The combat config for the player.
    pub combat_config: PlayerCombatConfig,
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            last_attack: Instant::now(),
            last_hit: Instant::now(),
            sprinting: false,
            sneaking: false,
            combat_config: PlayerCombatConfig::default(),
        }
    }
}

/// Multipliers that will be applied when the condition is true.
/// Values of 1.0 are the vanilla values.
pub struct CombatMultipliers {
    /// The multiplier when the player is sprinting.
    pub sprinting: f32,
    /// The multiplier when the player is not sprinting.
    pub not_sprinting: f32,
    /// The multiplier when the player is sneaking.
    pub sneaking: f32,
    /// The multiplier when the player is not sneaking.
    pub not_sneaking: f32,
    /// The multiplier when the player is falling down (not when the player is jumping).
    pub falling: f32,
    /// The multiplier when the player is on the ground.
    pub on_ground: f32,
    /// The multiplier when the player is in the air (e.g jumping up or falling).
    pub in_air: f32,
}

impl Default for CombatMultipliers {
    fn default() -> Self {
        Self {
            sprinting: 1.0,
            not_sprinting: 1.0,
            sneaking: 1.0,
            not_sneaking: 1.0,
            falling: 1.0,
            on_ground: 1.0,
            in_air: 1.0,
        }
    }
}

/// Contains configuration options mostly multipliers for the player.
/// They will usually not be changed during the game.
pub struct PlayerCombatConfig {
    /// The armor resistance of the player
    pub armor_resistance_multiplier: CombatMultipliers,
    /// The damage taken multiplier of the player.
    pub damage_taken_multiplier: CombatMultipliers,
    /// The armor durability multiplier of the player.
    pub armor_durability_multiplier: CombatMultipliers,
    /// The damage dealt multiplier of the player.
    pub damage_dealt_multiplier: CombatMultipliers,
    /// A multiplier for the penalty of the attack cooldown.
    /// 0.0 means the player deals full damage with a partially charged attack.
    pub attack_cooldown_penalty_multiplier: CombatMultipliers,
    /// The knockback taken multiplier of the player.
    pub knockback_taken_multiplier: CombatMultipliers,
    /// The knockback dealt multiplier of the player.
    pub knockback_multiplier: CombatMultipliers,
    /// The players critical hit damage multiplier.
    pub critical_hit_damage_multiplier: CombatMultipliers,
    /// The burn damage multiplier of the player (the damage the player takes).
    pub burn_damage_multiplier: CombatMultipliers,
    /// The players critical hit chance.
    pub critical_hit_chance: CombatMultipliers,
    /// Friendly fire damage dealt multiplier.
    pub friendly_fire_damage_multiplier: CombatMultipliers,
    /// Friendly fire damage taken multiplier.
    pub friendly_fire_damage_taken_multiplier: CombatMultipliers,
    /// If arrows will stick to the player.
    pub arrows_stick: bool,
    /// Teams considered friendly.
    pub friendly_teams: HashSet<u16>,
    // TODO: shield
}

impl Default for PlayerCombatConfig {
    fn default() -> Self {
        Self {
            armor_resistance_multiplier: CombatMultipliers::default(),
            damage_taken_multiplier: CombatMultipliers::default(),
            armor_durability_multiplier: CombatMultipliers::default(),
            damage_dealt_multiplier: CombatMultipliers::default(),
            attack_cooldown_penalty_multiplier: CombatMultipliers::default(),
            knockback_taken_multiplier: CombatMultipliers::default(),
            knockback_multiplier: CombatMultipliers::default(),
            critical_hit_damage_multiplier: CombatMultipliers::default(),
            burn_damage_multiplier: CombatMultipliers::default(),
            critical_hit_chance: CombatMultipliers {
                sprinting: 0.0,
                not_sprinting: 0.0,
                sneaking: 0.0,
                not_sneaking: 0.0,
                falling: 1.0,
                on_ground: 0.0,
                in_air: 0.0,
            },
            friendly_fire_damage_multiplier: CombatMultipliers::default(),
            friendly_fire_damage_taken_multiplier: CombatMultipliers::default(),
            friendly_teams: HashSet::new(),
            arrows_stick: false, // Usually disabled on pvp servers.
        }
    }
}

/// A Team component that is attached to entities that are part of a team.
#[derive(Component)]
pub struct Team(pub u16);

#[derive(QueryData)]
#[query_data(mutable)]
pub struct CombatQuery {
    pub client: &'static mut Client,
    pub entity_id: &'static EntityId,
    pub position: &'static Position,
    pub velocity: &'static mut Velocity,
    pub state: &'static CombatState,
    pub statuses: &'static mut EntityStatuses,
    pub held_item: &'static HeldItem,
    pub falling_state: &'static FallingState,
    pub equipment: &'static Equipment,
    pub team: Option<&'static Team>,
    pub stuck_arrow_count: &'static mut StuckArrowCount,
}
