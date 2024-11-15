use valence::{
    entity::{living::Health, EntityId},
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
    /// suppress the death event
    pub suppress_death_event: bool,
}

impl Default for TakesDamage {
    fn default() -> Self {
        Self {
            show_hurt: true,
            play_sound: true,
            damage_multiplier: 1.0,
            set_hp_after_death: 0.0,
            suppress_death_event: false,
        }
    }
}

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_event::<DeathEvent>()
            .add_systems(Update, damage_system);
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
