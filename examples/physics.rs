use bevy_time::TimePlugin;
use physics::{Acceleration, BlockCollisionConfig, Drag, PhysicsPlugin, SpeedLimit};
use valence::entity::chicken::ChickenEntityBundle;
use valence::entity::entity::NoGravity;
use valence::entity::Velocity;
use valence::prelude::*;

const SPAWN_Y: i32 = 64;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TimePlugin)
        .add_systems(Startup, setup)
        .add_plugins(PhysicsPlugin)
        .add_systems(
            Update,
            (
                init_clients,
                despawn_disconnected_clients,
                on_player_sneak_click,
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

    let mut y = SPAWN_Y;
    for z in 5..25 {
        for y_inner in 0..y {
            layer.chunk.set_block([5, y_inner, z], BlockState::STONE);
        }
        y += 1;
    }

    commands.spawn(layer);
}

#[allow(clippy::type_complexity)]
fn init_clients(
    mut clients: Query<
        (
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
    }
}

fn on_player_sneak_click(
    mut commands: Commands,
    query: Query<(&Position, &Look, &EntityLayerId), With<Client>>,
    mut events: EventReader<SneakEvent>,
) {
    for event in events.read() {
        if event.state != SneakState::Start {
            continue;
        }

        let Ok((pos, look, layer_id)) = query.get(event.client) else {
            continue;
        };

        let yaw = look.yaw.to_radians();
        let pitch = look.pitch.to_radians();

        let direction = Vec3::new(
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        );

        commands
            .spawn(ChickenEntityBundle {
                position: Position(
                    pos.0 + DVec3::new(0.0, 1.0, 0.0) + (direction * 2.0).as_dvec3(),
                ),
                velocity: Velocity(direction * 20.0),
                entity_no_gravity: NoGravity(true),
                layer: *layer_id,

                ..Default::default()
            })
            .insert(Acceleration(Vec3::new(0.0, -20.0, 0.0)))
            .insert(Drag(Vec3::new(0.99 / 20.0, 0.99 / 20.0, 0.99 / 20.0)))
            .insert(SpeedLimit(100.0))
            .insert(BlockCollisionConfig::default());
    }
}
