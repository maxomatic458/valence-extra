use bevy_time::TimePlugin;
// #![cfg(feature = "chat")]
use combat::{CombatPlugin, CombatState};
use fall_damage::{FallDamagePlugin, FallingState};
use physics::{Acceleration, BlockCollisionConfig, PhysicsPlugin, StopOnBlockCollision};
use utils::{
    damage::{DamageEvent, DamagePlugin, TakesDamage},
    item_values::CombatSystem,
};
use valence::{
    entity::{zombie::ZombieEntityBundle, EntityStatuses},
    equipment::EquipmentInventorySync,
    prelude::*,
};
const SPAWN_Y: i32 = 64;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FallDamagePlugin)
        .add_plugins(TimePlugin)
        .add_systems(Startup, setup)
        .add_plugins(PhysicsPlugin)
        .add_plugins(DamagePlugin)
        .add_plugins(CombatPlugin)
        .add_systems(
            Update,
            (
                init_clients,
                despawn_disconnected_clients,
                on_damage,
                on_player_sneak,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
) {
    let mut layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);

    for z in -5..5 {
        for x in -5..5 {
            layer.chunk.insert_chunk([x, z], UnloadedChunk::new());
        }
    }

    for z in -25..25 {
        for x in -25..25 {
            layer
                .chunk
                .set_block([x, SPAWN_Y, z], BlockState::GRASS_BLOCK);
        }
    }

    let id = commands.spawn(layer).id();

    commands
        .spawn(ZombieEntityBundle {
            position: Position([3.0, f64::from(SPAWN_Y) + 1.0, 3.0].into()),
            layer: EntityLayerId(id),

            ..Default::default()
        })
        .insert(FallingState::default())
        .insert(TakesDamage {
            damage_multiplier: 0.0,

            ..Default::default()
        })
        .insert(BlockCollisionConfig::default())
        .insert(Acceleration(Vec3::new(0.0, -32.0, 0.0)))
        .insert(StopOnBlockCollision::ground())
        .insert(CombatState::default())
        .insert(EntityStatuses::default())
        .insert(Equipment::default());
}

#[allow(clippy::type_complexity)]
fn init_clients(
    mut commands: Commands,
    mut clients: Query<
        (
            Entity,
            &mut Client,
            &mut Position,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut GameMode,
            &mut Inventory,
        ),
        Added<Client>,
    >,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    for (
        player_ent,
        mut client,
        mut pos,
        mut layer_id,
        mut visible_chunk_layer,
        mut visible_entity_layers,
        mut game_mode,
        mut inventory,
    ) in &mut clients
    {
        let layer = layers.single();

        pos.0 = [0.0, f64::from(SPAWN_Y) + 1.0, 0.0].into();
        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        visible_entity_layers.0.insert(layer);
        *game_mode = GameMode::Survival;

        commands
            .entity(player_ent)
            .insert(CombatState::default())
            .insert(FallingState::new(pos.0))
            .insert(EquipmentInventorySync);

        inventory.set_slot(36, ItemStack::new(ItemKind::DiamondSword, 1, None));
        inventory.set_slot(37, ItemStack::new(ItemKind::DiamondPickaxe, 1, None));
        inventory.set_slot(38, ItemStack::new(ItemKind::DiamondAxe, 1, None));

        client.send_chat_message("Sneak to switch the combat system (1.8/1.9+)");
    }
}

fn on_damage(mut events: EventReader<DamageEvent>, mut clients: Query<&mut Client>) {
    for event in events.read() {
        if let Some(attacker) = event.attacker {
            let Ok(mut client) = clients.get_mut(attacker) else {
                continue;
            };

            client.send_chat_message(format!("Damage: {}", event.damage));
        }
    }
}

fn on_player_sneak(
    mut query: Query<(&mut Client, &mut CombatState), With<Client>>,
    mut events: EventReader<SneakEvent>,
) {
    for event in events.read() {
        if event.state != SneakState::Start {
            continue;
        }

        let Ok((mut client, mut combat_state)) = query.get_mut(event.client) else {
            continue;
        };

        if combat_state.combat_config.combat_system == CombatSystem::Old {
            combat_state.combat_config.combat_system = CombatSystem::New;
            combat_state.combat_config.attack_cooldown_multiplier = Some(1.0);
            client.set_action_bar("Switched to new combat system");
        } else {
            combat_state.combat_config.combat_system = CombatSystem::Old;
            combat_state.combat_config.attack_cooldown_multiplier = None;
            client.set_action_bar("Switched to old combat system");
        }
    }
}
