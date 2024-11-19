// #![cfg(feature = "chat")]

use bevy_time::TimePlugin;
use fall_damage::{FallDamagePlugin, FallingState};
use utils::damage::{DamagePlugin, TakesDamage};
use valence::prelude::*;

const SPAWN_Y: i32 = 64;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_plugins(TimePlugin)
        .add_plugins(DamagePlugin)
        .add_plugins(FallDamagePlugin)
        .add_systems(Update, (init_clients, despawn_disconnected_clients))
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

    let mut y = SPAWN_Y;
    for z in 5..25 {
        layer.chunk.set_block([5, y, z], BlockState::STONE);
        y += 1;
    }

    commands.spawn(layer);
}

#[allow(clippy::type_complexity)]
fn init_clients(
    mut commands: Commands,
    mut clients: Query<
        (
            Entity,
            &mut Position,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut GameMode,
        ),
        Added<Client>,
    >,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    for (
        player_ent,
        mut pos,
        mut layer_id,
        mut visible_chunk_layer,
        mut visible_entity_layers,
        mut game_mode,
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
            .insert(TakesDamage {
                set_hp_after_death: 20.0,
                ..Default::default()
            })
            .insert(FallingState::new(pos.0));
    }
}
