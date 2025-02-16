use bevy_time::TimePlugin;
use building::{BuildPlugin, BuildState};
use physics::{
    Acceleration, BlockCollisionConfig, Drag, EntityCollisionConfig, PhysicsPlugin, SpeedLimit,
};
use valence::entity::chicken::ChickenEntityBundle;
use valence::entity::entity::NoGravity;
use valence::entity::Velocity;
use valence::inventory::player_inventory::PlayerInventory;
use valence::prelude::*;

const SPAWN_Y: i32 = 64;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TimePlugin)
        .add_systems(Startup, setup)
        .add_plugins(PhysicsPlugin)
        .add_plugins(BuildPlugin)
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
    commands.spawn(layer);
}

#[allow(clippy::type_complexity)]
fn init_clients(
    mut commands: Commands,
    mut clients: Query<
        (
            Entity,
            &mut Position,
            &mut Inventory,
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
        entity,
        mut pos,
        mut inventory,
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
        inventory.set_slot(
            PlayerInventory::hotbar_to_slot(4),
            ItemStack::new(ItemKind::Stone, 64, None),
        );

        // Make the player be able to build.
        commands.entity(entity).insert(BuildState::default());
        // Make the player block placements. (so the player cant place blocks in itself).
        // TODO: make this be the [`physics::BlockCollisionConfig`]
        commands
            .entity(entity)
            .insert(BlockCollisionConfig::default());
    }
}
