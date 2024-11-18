use std::time::Duration;

use bevy_time::{Time, Timer, TimerMode};
use valence::{
    entity::{entity::Flags, living::Health, EntityId},
    prelude::*,
    protocol::{packets::play::EntityDamageS2c, sound::SoundCategory, Sound, VarInt, WritePacket},
    Layer,
};

/// An event that will be fired if an entity takes damage.
#[derive(Event)]
pub struct DamageEvent {
    pub victim: Entity,
    pub attacker: Option<Entity>,
    pub damage: f32,
}

#[derive(Event)]
pub struct StartBurningEvent {
    pub victim: Entity,
    pub attacker: Option<Entity>,
    pub duration: Duration,
    pub damage_per_second: f32,
}

/// Marker component for entities that are on fire.
#[derive(Component)]
struct OnFire;

/// An event that will be fired if an entity dies.
#[derive(Event)]
pub struct DeathEvent {
    pub victim: Entity,
    pub attacker: Option<Entity>,
}

/// This component will be added to entities that register damage with the [`DamageEvent`]
#[derive(Component)]
pub struct TakesDamage {
    /// If the hurt animation should be shown when the player is hit (the player will turn red for a others).
    pub show_hurt: bool,
    /// If the damage sound should be played when the player is hit.
    pub play_sound: bool,
    /// The damage multiplier for the entity.
    pub damage_multiplier: f32,
    /// Set the health of the entity to this value after the entity dies.
    /// If the entity dies (through the [`DamageEvent`]), the an [`DeathEvent`] will be fired and the health will be set to this value.
    /// If the value is > 0, this will prevent the minecraft death screen (allowing for custom death/respawn logic).
    pub set_hp_after_death: f32,
    /// Suppress the death event.
    pub suppress_death_event: bool,

    /// Show flames when the entity is burning.
    pub show_burning: bool,
    /// Burn duration multiplier.
    pub burn_duration_multiplier: f32,
    /// Burn damage multiplier.
    pub burn_damage_multiplier: f32,
}

#[derive(Component)]
struct BurnTimer {
    pub second_timer: Timer,
    pub full_timer: Timer,
    seconds_left: u32,
    attacker: Option<Entity>,
    damage_per_second: f32,
}

impl BurnTimer {
    pub fn new(duration: Duration, attacker: Option<Entity>, damage_per_second: f32) -> Self {
        Self {
            second_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
            full_timer: Timer::new(duration, TimerMode::Once),
            seconds_left: duration.as_secs() as u32,
            attacker,
            damage_per_second,
        }
    }
}

impl Default for TakesDamage {
    fn default() -> Self {
        Self {
            show_hurt: true,
            play_sound: true,
            damage_multiplier: 1.0,
            set_hp_after_death: 0.0,
            suppress_death_event: false,
            show_burning: true,
            burn_duration_multiplier: 1.0,
            burn_damage_multiplier: 1.0,
        }
    }
}

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_event::<DeathEvent>()
            .add_event::<StartBurningEvent>()
            .add_systems(Update, (damage_system, burn_system));
    }
}

fn damage_system(
    mut events: EventReader<DamageEvent>,
    mut event_writer: EventWriter<DeathEvent>,
    mut query: Query<(&mut Health, &TakesDamage, &Position, &EntityId)>,
    mut layer: Query<&mut ChunkLayer>,
) {
    for events in events.read() {
        for (mut health, takes_damage, position, entity_id) in query.iter_mut() {
            if health.0 <= 0.0 {
                continue;
            }

            let entity_id: VarInt = entity_id.get().into();

            let damage = events.damage * takes_damage.damage_multiplier;
            health.0 -= damage;

            let mut layer = layer.single_mut();

            if takes_damage.show_hurt {
                layer
                    .view_writer(position.0)
                    .write_packet(&EntityDamageS2c {
                        entity_id,
                        source_type_id: 1.into(),
                        source_cause_id: 0.into(),
                        source_direct_id: 0.into(),
                        source_pos: Some(position.0),
                    });
            }

            if health.0 <= 0.0 {
                if takes_damage.play_sound {
                    layer.play_sound(
                        Sound::EntityPlayerDeath,
                        SoundCategory::Player,
                        position.0,
                        1.0,
                        1.0,
                    );
                }

                if !takes_damage.suppress_death_event {
                    event_writer.send(DeathEvent {
                        victim: events.victim,
                        attacker: events.attacker,
                    });
                }

                health.0 = takes_damage.set_hp_after_death;
            } else if takes_damage.play_sound {
                layer.play_sound(
                    Sound::EntityPlayerHurt,
                    SoundCategory::Player,
                    position.0,
                    1.0,
                    1.0,
                );
            }
        }
    }
}

fn burn_system(
    mut commands: Commands,
    mut events: EventReader<StartBurningEvent>,
    mut query: Query<(Entity, &TakesDamage, Option<&mut BurnTimer>, &mut Flags)>,
    mut damage_writer: EventWriter<DamageEvent>,
    time: Res<Time>,
) {
    for (victim, takes_damage, burn_timer, mut flags) in query.iter_mut() {
        if let Some(mut burn_timer) = burn_timer {
            if !burn_timer.full_timer.tick(time.delta()).finished() {
                if burn_timer.second_timer.tick(time.delta()).finished() {
                    burn_timer.seconds_left -= 1;
                    damage_writer.send(DamageEvent {
                        victim,
                        attacker: burn_timer.attacker,
                        damage: burn_timer.damage_per_second * takes_damage.burn_damage_multiplier,
                    });
                }
            } else {
                commands.entity(victim).remove::<OnFire>();
                commands.entity(victim).remove::<BurnTimer>();
                flags.set_on_fire(false);
            }
        }
    }

    for event in events.read() {
        for (victim, takes_damage, _, mut flags) in query.iter_mut() {
            let duration = event
                .duration
                .mul_f32(takes_damage.burn_duration_multiplier);
            let burn_timer = BurnTimer::new(duration, event.attacker, event.damage_per_second);
            commands.entity(victim).insert(burn_timer);
            commands.entity(victim).insert(OnFire);

            flags.set_on_fire(true);
        }
    }
}
