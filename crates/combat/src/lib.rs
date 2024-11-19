use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use bevy_ecs::query::QueryData;
use calculations::damage_after_armor;
use fall_damage::FallingState;
use utils::{
    damage::{DamageEvent, StartBurningEvent},
    enchantments::{Enchantment, ItemStackEnchantmentsExt},
    item_values::{CombatSystem, EquipmentExt},
    ItemKindExt,
};
use valence::{
    entity::{
        attributes::{EntityAttribute, EntityAttributes},
        living::StuckArrowCount,
        EntityId, EntityStatuses, Velocity,
    },
    hand_swing::HandSwingEvent,
    inventory::{HeldItem, UpdateSelectedSlotEvent},
    prelude::*,
};

pub mod calculations;

const BASE_HIT_COOLDOWN: Duration = Duration::from_millis(500);

/// Attached to every player that participates in combat.
#[derive(Component)]
pub struct CombatState {
    /// Last time the player hit another entity.
    pub last_hit: Instant,
    /// The last time the player was hit by another entity.
    pub last_got_hit: Instant,
    /// Last time the player switched the item or attacked (used for attack cooldown, 1.9+).
    pub last_attack: Instant,
    /// The player is sprinting.
    pub sprinting: bool,
    /// The player is sneaking.
    pub sneaking: bool,
    /// The combat config for the player.
    pub combat_config: PlayerCombatConfig,
    /// The player is currently blocking with a shield.
    pub blocking: bool,
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            last_hit: Instant::now(),
            last_got_hit: Instant::now(),
            last_attack: Instant::now(),
            sprinting: false,
            sneaking: false,
            combat_config: PlayerCombatConfig::default(),
            blocking: false,
        }
    }
}

/// Contains configuration options mostly multipliers for the player.
/// They will usually not be changed during the game.
pub struct PlayerCombatConfig {
    /// The combat system that will be used to determine the weapon damage.
    /// This only affects the damage, not the actual cooldown, change [`Self::attack_cooldown_multiplier`] for that.
    ///
    /// [`CombatSystem::Old`] is the 1.8 combat system.
    ///
    /// [`CombatSystem::New`] is the 1.9+ combat system.
    pub combat_system: CombatSystem,
    /// How many arrows can be in the player at once.
    pub arrows_stick: u8,
    /// Teams considered friendly.
    pub friendly_teams: HashSet<u16>,
    /// The minimum time between two attacks. (This is not the attack cooldown, but the minimum time before another attack can be registered).
    pub hit_cooldown: Duration,
    /// The attack cooldown of the play (as in 1.9+).
    ///
    /// If `None`, no attack cooldown will be applied.
    ///
    /// If `Some`, then the multiplier will be applied to the 1.9+ attack cooldown.
    pub attack_cooldown_multiplier: Option<f32>,

    /// Multiplier for armor points.
    pub armor_points_multiplier: f32,
    /// Multiplier for armor toughness.
    pub armor_toughness_multiplier: f32,
    /// Multiplier for the knockback resistance applied by armor.
    pub armor_knockback_resistance_multiplier: f32,

    /// Horizontal knockback the player deals.
    pub horizontal_knockback: PlayerStateDependantValue,
    /// Vertical knockback the player deals.
    pub vertical_knockback: PlayerStateDependantValue,

    /// Multiplier of the horizontal knockback the player takes.
    pub horizontal_knockback_received_multiplier: PlayerStateDependantValue,
    /// Multiplier of the vertical knockback the player takes.
    pub vertical_knockback_received_multiplier: PlayerStateDependantValue,

    /// The random chance of a critical hit (0.0 - 1.0).
    pub random_critical_hit_chance: PlayerStateDependantValue,
    /// The random chance of a critical hit while falling (0.0 - 1.0), vanilla is 100%.
    pub critical_hit_chance_falling: f32,
    /// The damage multiplier of a critical hit.
    pub critical_hit_damage_multiplier: f32,

    /// The damage multiplier of the player.
    pub damage_multiplier: PlayerStateDependantValue,
    /// Fire damage multiplier of the player.
    pub fire_damage_multiplier: PlayerStateDependantValue,
    /// Fire duration multiplier of the player.
    pub fire_duration_multiplier: PlayerStateDependantValue,

    /// The damage multiplier the player takes.
    pub damage_taken_multiplier: PlayerStateDependantValue,

    /// Multiplier for damage dealt to entities considered friendly.
    pub friendly_fire_damage_multiplier: f32,
    /// Multiplier for damage taken from entities considered friendly.
    pub friendly_fire_damage_taken_multiplier: f32,

    /// The formula that should be used to calculate the received damage after armor.
    ///
    /// The parameters are: `damage`, `armor_points`, `toughness`.
    pub armor_formula: fn(f32, f32, f32) -> f32,

    /// Attack cooldown damage multiplier for weapon damage formula
    ///
    /// The parameters are: `weapon_attack_speed`, `last_attack`.
    pub damage_cooldown_formula_base_damage: fn(f32, Instant) -> f32,

    /// Attack cooldown damage multiplier for enchantments formula
    ///
    /// The parameters are: `weapon_attack_speed`, `last_attack`.
    pub damage_cooldown_enchantment_formula: fn(f32, Instant) -> f32,

    /// The configuration of combat relevant enchantments.
    pub enchantment_config: CombatEnchantmentConfig,
}

/// The current state of the player's movement.
enum PlayerMovementState {
    Sprinting,
    Sneaking,
    InAir,
    None,
}

/// Values that depend on the current state of the player.
pub struct PlayerStateDependantValue {
    pub base: f32,
    pub sprinting: f32,
    pub sneaking: f32,
    pub in_air: f32,
}

impl PlayerStateDependantValue {
    pub fn always(value: f32) -> Self {
        Self {
            base: value,
            sprinting: value,
            sneaking: value,
            in_air: value,
        }
    }
    /// Get the current value based on the player's state.
    /// The priority is: in_air > sprinting > sneaking > base.
    fn current(&self, movement_state: &PlayerMovementState) -> f32 {
        match movement_state {
            PlayerMovementState::InAir => self.in_air,
            PlayerMovementState::Sprinting => self.sprinting,
            PlayerMovementState::Sneaking => self.sneaking,
            PlayerMovementState::None => self.base,
        }
    }
}

pub struct CombatEnchantmentConfig {
    /// The formula to calculate the damage after applying the sharpness enchantment.
    ///
    /// The parameters are: `weapon_base_damage`, `sharpness_level`.
    ///
    /// If this is `None`, the enchantment will not be usable by the player.
    pub sharpness_formula: Option<fn(f32, u32) -> f32>,
    /// The formula to calculate the knockback after applying the knockback enchantment.
    ///
    /// The parameters are: `base_knockback_vector`, `knockback_level`.
    ///
    /// If this is `None`, the enchantment will not be usable by the player.
    pub knockback_formula: Option<fn(Vec3, u32) -> Vec3>,
    /// The formula to calculate the burn time and damage per second after applying the fire aspect enchantment.
    ///
    /// The parameters are: `fire_aspect_level`.
    ///
    /// If this is `None`, the enchantment will not be usable by the player.
    pub fire_aspect_formula: Option<fn(u32) -> (Duration, f32)>,
    /// The formula to calculate the burn time and damage per second after applying the flame enchantment.
    ///
    /// The parameters are: `fire_aspect_level`.
    ///
    /// If this is `None`, the enchantment will not be usable by the player.
    pub flame_formula: Option<fn(u32) -> (Duration, f32)>,
    /// The formula to calculate the damage after applying the power enchantment.
    ///
    /// The parameters are: `base_arrow_damage`, `power_level`.
    ///
    /// If this is `None`, the enchantment will not be usable by the player.
    pub power_formula: Option<fn(f32, u32) -> f32>,
    /// The formula to calculate the knockback after applying the punch enchantment.
    ///
    /// The parameters are: `base_knockback_vector`, `punch_level`.
    ///
    /// If this is `None`, the enchantment will not be usable by the player.
    pub punch_formula: Option<fn(Vec3, u32) -> Vec3>,
    // TODO: thorns,
}

impl Default for PlayerCombatConfig {
    fn default() -> Self {
        Self {
            combat_system: CombatSystem::Old,
            arrows_stick: 0,
            friendly_teams: HashSet::new(),
            hit_cooldown: BASE_HIT_COOLDOWN,
            attack_cooldown_multiplier: None,
            armor_points_multiplier: 1.0,
            armor_toughness_multiplier: 1.0,
            armor_knockback_resistance_multiplier: 1.0,
            horizontal_knockback: PlayerStateDependantValue {
                base: 0.4,
                sprinting: 0.8,
                sneaking: 0.4,
                in_air: 0.4,
            },
            vertical_knockback: PlayerStateDependantValue {
                base: 0.36,
                sprinting: 0.42,
                sneaking: 0.36,
                in_air: 0.36,
            },
            horizontal_knockback_received_multiplier: PlayerStateDependantValue {
                base: 1.0,
                sprinting: 1.0,
                sneaking: 1.0,
                in_air: 0.6,
            },
            vertical_knockback_received_multiplier: PlayerStateDependantValue {
                base: 1.0,
                sprinting: 1.0,
                sneaking: 1.0,
                in_air: 0.8,
            },
            random_critical_hit_chance: PlayerStateDependantValue::always(0.0),
            critical_hit_chance_falling: 1.0,
            critical_hit_damage_multiplier: 1.5,
            damage_multiplier: PlayerStateDependantValue::always(1.0),
            damage_taken_multiplier: PlayerStateDependantValue::always(1.0),
            fire_damage_multiplier: PlayerStateDependantValue::always(1.0),
            fire_duration_multiplier: PlayerStateDependantValue::always(1.0),
            friendly_fire_damage_multiplier: 0.0,
            friendly_fire_damage_taken_multiplier: 0.0,
            armor_formula: calculations::damage_after_armor,
            enchantment_config: CombatEnchantmentConfig {
                sharpness_formula: Some(calculations::enchant_sharpness_damage),
                knockback_formula: Some(calculations::enchant_knockback),
                fire_aspect_formula: Some(calculations::enchant_fire_aspect),
                flame_formula: Some(calculations::enchant_flame),
                power_formula: Some(calculations::enchant_power_damage),
                punch_formula: Some(calculations::enchant_punch),
            },
            damage_cooldown_formula_base_damage: calculations::attack_cooldown_base_damage,
            damage_cooldown_enchantment_formula: calculations::attack_cooldown_enchantment_damage,
        }
    }
}

struct EnchantmentValues {
    damage: f32,
    knockback: Vec3,
    /// The burn time and damage per second.
    burn: Option<(Duration, f32)>,
}

/// Applies the enchantments and returns the new values.
fn apply_enchantments(
    mut base_damage: f32,
    mut base_knockback: Vec3,
    enchantments: HashMap<Enchantment, u32>,
    enchantment_config: &CombatEnchantmentConfig,
) -> EnchantmentValues {
    let mut burn = None;

    for (enchant, level) in enchantments {
        match enchant {
            Enchantment::Sharpness => {
                if let Some(formula) = &enchantment_config.sharpness_formula {
                    base_damage = formula(base_damage, level);
                }
            }
            Enchantment::Knockback => {
                if let Some(formula) = &enchantment_config.knockback_formula {
                    base_knockback = formula(base_knockback, level);
                }
            }
            Enchantment::FireAspect => {
                if let Some(formula) = &enchantment_config.fire_aspect_formula {
                    burn = Some(formula(level));
                }
            }
            Enchantment::Flame => {
                if let Some(formula) = &enchantment_config.flame_formula {
                    burn = Some(formula(level));
                }
            }
            Enchantment::Power => {
                if let Some(formula) = &enchantment_config.power_formula {
                    base_damage = formula(base_damage, level);
                }
            }
            Enchantment::Punch => {
                if let Some(formula) = &enchantment_config.punch_formula {
                    base_knockback = formula(base_knockback, level);
                }
            }
            _ => {}
        }
    }

    EnchantmentValues {
        damage: base_damage,
        knockback: base_knockback,
        burn,
    }
}

/// A Team component that is attached to entities that are part of a team.
#[derive(Component, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Team(pub u16);

#[derive(QueryData)]
#[query_data(mutable)]
struct CombatQuery {
    client: Option<&'static mut Client>,
    entity_id: &'static EntityId,
    position: &'static Position,
    velocity: &'static mut Velocity,
    state: &'static mut CombatState,
    statuses: &'static mut EntityStatuses,
    // To retrieve the weapon used.
    inventory: Option<&'static Inventory>,
    // Held item is optional so we can add the CombatQuery to NPCs as well.
    held_item: Option<&'static HeldItem>,
    falling_state: &'static FallingState,
    equipment: &'static Equipment,
    team: Option<&'static Team>,
    stuck_arrow_count: Option<&'static mut StuckArrowCount>,
    // Used for the attack cooldown
    attributes: &'static mut EntityAttributes,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                combat_system,
                update_last_attack_on_item_switch,
                on_hand_swing,
            ),
        );
    }
}

fn combat_system(
    mut query: Query<CombatQuery>,
    mut damage_event_writer: EventWriter<DamageEvent>,
    mut start_burn_event_writer: EventWriter<StartBurningEvent>,
    mut sprinting_events: EventReader<SprintEvent>,
    mut sneaking_events: EventReader<SneakEvent>,
    mut interact_entity_events: EventReader<InteractEntityEvent>,
) {
    for &SprintEvent { client, state } in sprinting_events.read() {
        if let Ok(mut client) = query.get_mut(client) {
            client.state.sprinting = state == SprintState::Start;
        }
    }

    for &SneakEvent { client, state } in sneaking_events.read() {
        if let Ok(mut client) = query.get_mut(client) {
            client.state.sneaking = state == SneakState::Start;
        }
    }

    for &InteractEntityEvent {
        client: attacker_ent,
        entity: victim_ent,
        interact,
        ..
    } in interact_entity_events.read()
    {
        if !matches!(interact, EntityInteraction::Attack) {
            continue;
        }

        if attacker_ent == victim_ent {
            continue;
        }

        let Ok([mut attacker, mut victim]) = query.get_many_mut([attacker_ent, victim_ent]) else {
            continue;
        };

        if attacker.state.last_hit.elapsed() < attacker.state.combat_config.hit_cooldown {
            continue;
        }

        let attacker_config = &attacker.state.combat_config;
        let victim_config = &victim.state.combat_config;

        let attacker_state = match (
            attacker.state.sprinting,
            attacker.state.sneaking,
            attacker.falling_state.falling,
        ) {
            (true, _, _) => PlayerMovementState::Sprinting,
            (_, true, _) => PlayerMovementState::Sneaking,
            (_, _, true) => PlayerMovementState::InAir,
            _ => PlayerMovementState::None,
        };

        let victim_state = match (
            victim.state.sprinting,
            victim.state.sneaking,
            victim.falling_state.falling,
        ) {
            (true, _, _) => PlayerMovementState::Sprinting,
            (_, true, _) => PlayerMovementState::Sneaking,
            (_, _, true) => PlayerMovementState::InAir,
            _ => PlayerMovementState::None,
        };

        let direction = (victim.position.0 - attacker.position.0)
            .normalize()
            .as_vec3();

        let weapon = match (attacker.held_item, attacker.inventory) {
            (Some(held_item), Some(inventory)) => inventory.slot(held_item.slot()),
            _ => return,
        };

        let knockback_xz = attacker_config
            .horizontal_knockback
            .current(&attacker_state);
        let knockback_y = attacker_config.vertical_knockback.current(&attacker_state);

        // TODO: set based on tick rate
        // TODO: this is not accurate
        let knockback = Vec3::new(
            direction.x * knockback_xz * 20.0,
            knockback_y * 20.0,
            direction.z * knockback_xz * 20.0,
        );

        let weapon_echants = weapon.enchantments();
        let mut base_damage = weapon.item.attack_damage(&attacker_config.combat_system);

        if let Some(cooldown_multiplier) = &attacker_config.attack_cooldown_multiplier {
            base_damage = base_damage
                * (attacker_config.damage_cooldown_formula_base_damage)(
                    weapon.item.attack_speed(),
                    attacker.state.last_attack,
                )
                * cooldown_multiplier;
        }

        let EnchantmentValues {
            mut damage,
            mut knockback,
            burn,
        } = apply_enchantments(
            base_damage,
            knockback,
            weapon_echants,
            &attacker_config.enchantment_config,
        );

        if let Some((burn_time, burn_dps)) = burn {
            let burn_event = StartBurningEvent {
                victim: victim_ent,
                attacker: Some(attacker_ent),
                duration: burn_time.mul_f32(
                    attacker_config
                        .fire_duration_multiplier
                        .current(&attacker_state),
                ),
                damage_per_second: burn_dps
                    * attacker_config
                        .fire_damage_multiplier
                        .current(&attacker_state),
            };

            start_burn_event_writer.send(burn_event);
        }

        // let enchantment_extra_dmg = damage - base_damage;

        // TODO: add this back

        // TODO: im not sure if this is actually applied after the base damage.
        // if let Some(cooldown_multiplier) = &attacker_config.attack_cooldown_multiplier {
        //     damage = damage
        //         * (attacker_config.damage_cooldown_enchantment_formula)(
        //             weapon.item.attack_speed(),
        //             attacker.state.last_attack,
        //         )
        //         * cooldown_multiplier;
        // }

        damage *= attacker_config.damage_multiplier.current(&attacker_state);

        damage = damage_after_armor(
            damage,
            victim.equipment.armor_points() * victim_config.armor_points_multiplier,
            victim.equipment.armor_toughness() * victim_config.armor_toughness_multiplier,
        );

        damage *= victim_config.damage_taken_multiplier.current(&victim_state);

        if let (Some(attacker_team), Some(victim_team)) = (attacker.team, victim.team) {
            if attacker_team == victim_team {
                damage *= attacker_config.friendly_fire_damage_multiplier;
                damage *= victim_config.friendly_fire_damage_taken_multiplier;
            }
        }

        if attacker_config
            .random_critical_hit_chance
            .current(&attacker_state)
            + if attacker.falling_state.falling {
                attacker_config.critical_hit_chance_falling
            } else {
                0.0
            }
            > rand::random::<f32>()
        {
            damage *= attacker_config.critical_hit_damage_multiplier;
        }

        let knockback_resistance = victim.equipment.knockback_resistance()
            * victim_config.armor_knockback_resistance_multiplier;

        knockback.x *= 1.0 - knockback_resistance;
        knockback.z *= 1.0 - knockback_resistance;
        // Is the y knockback ignored?
        knockback.y *= 1.0 - knockback_resistance;

        let knockback_received_xz_mult = victim_config
            .horizontal_knockback_received_multiplier
            .current(&victim_state);

        let knockback_received_y_mult = victim_config
            .vertical_knockback_received_multiplier
            .current(&victim_state);

        knockback.x *= knockback_received_xz_mult;
        knockback.z *= knockback_received_xz_mult;
        knockback.y *= knockback_received_y_mult;

        if let Some(mut client) = victim.client {
            client.set_velocity(knockback);
        } else {
            victim.velocity.0 += knockback;
        }

        let now = Instant::now();

        attacker.state.last_hit = now;
        attacker.state.last_attack = now;
        victim.state.last_got_hit = now;

        damage_event_writer.send(DamageEvent {
            victim: victim_ent,
            attacker: Some(attacker_ent),
            damage,
        });
    }
}

// TODO: new combat system is has not been tested i think

// If the player changes their hotbar slot, update the last attack time,
// this is the vanilla behavior.
fn update_last_attack_on_item_switch(
    mut query: Query<CombatQuery>,
    mut events: EventReader<UpdateSelectedSlotEvent>,
) {
    for event in events.read() {
        if let Ok(mut combat_query) = query.get_mut(event.client) {
            combat_query.state.last_attack = Instant::now();

            if let Some(cooldown_multiplier) =
                &combat_query.state.combat_config.attack_cooldown_multiplier
            {
                if let (Some(held_item), Some(inventory)) =
                    (combat_query.held_item, combat_query.inventory)
                {
                    let held_item = inventory.slot(held_item.slot());
                    let attack_speed = held_item.item.attack_speed() * cooldown_multiplier;

                    combat_query
                        .attributes
                        .set_base_value(EntityAttribute::GenericAttackSpeed, attack_speed as f64);
                }
            }
        }
    }

    for mut state in query.iter_mut() {
        if let (Some(held_item), Some(inventory)) = (state.held_item, state.inventory) {
            let held_item_slot = held_item.slot();

            if inventory.changed & (1 << held_item_slot) != 0 {
                state.state.last_attack = Instant::now();

                if let Some(cooldown_multiplier) =
                    &state.state.combat_config.attack_cooldown_multiplier
                {
                    let held_item = inventory.slot(held_item.slot());
                    let attack_speed = held_item.item.attack_speed() * cooldown_multiplier;

                    state
                        .attributes
                        .set_base_value(EntityAttribute::GenericAttackSpeed, attack_speed as f64);
                }
            }
        }
    }
}

fn on_hand_swing(mut query: Query<CombatQuery>, mut events: EventReader<HandSwingEvent>) {
    for event in events.read() {
        if let Ok(mut combat_query) = query.get_mut(event.client) {
            combat_query.state.last_attack = Instant::now();
        }
    }
}
