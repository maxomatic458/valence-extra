use bevy_time::TimePlugin;
use physics::{
    Acceleration, BlockCollisionConfig, Drag, EntityBlockCollisionEvent, EntityCollisionConfig,
    EntityEntityCollisionEvent, PhysicsPlugin, SpeedLimit,
};
use valence::entity::entity::NoGravity;
use valence::entity::pig::PigEntityBundle;
use valence::entity::snowball::SnowballEntityBundle;
use valence::entity::Velocity;
use valence::interact_item::InteractItemEvent;
use valence::inventory::player_inventory::PlayerInventory;
use valence::prelude::*;
use valence::protocol::sound::{Sound, SoundCategory};

const SPAWN_Y: i32 = 64;

/// Marker component for the target.
#[derive(Component)]
struct TargetMarker;

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
                on_player_right_click,
                on_entity_block_collision,
                on_entity_entity_collision,
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

    let layer_id = commands.spawn(layer).id();

    commands
        .spawn(PigEntityBundle {
            position: Position([0.0, f64::from(SPAWN_Y) + 1.0, 0.0].into()),
            layer: valence::prelude::EntityLayerId(layer_id),
            ..Default::default()
        })
        .insert(EntityCollisionConfig::default())
        .insert(TargetMarker);
}

#[allow(clippy::type_complexity)]
fn init_clients(
    mut clients: Query<
        (
            &mut Inventory,
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
        mut inventory,
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
        inventory.set_slot(
            PlayerInventory::hotbar_to_slot(4),
            ItemStack::new(ItemKind::IronIngot, 1, None),
        );
    }
}

fn on_player_right_click(
    mut commands: Commands,
    mut query: Query<(&mut Client, &Position, &Look, &EntityLayerId)>,
    mut events: EventReader<InteractItemEvent>,
) {
    for event in events.read() {
        let Ok((mut client, pos, look, layer_id)) = query.get_mut(event.client) else {
            continue;
        };

        let yaw = look.yaw.to_radians();
        let pitch = look.pitch.to_radians();

        let direction = Vec3::new(
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        );

        client.play_sound(
            Sound::EntityArrowShoot,
            SoundCategory::Neutral,
            pos.0,
            1.0,
            1.0,
        );

        commands
            .spawn(SnowballEntityBundle {
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
            .insert(EntityCollisionConfig::default())
            .insert(BlockCollisionConfig::default());
    }
}

fn on_entity_block_collision(
    mut commands: Commands,
    mut events: EventReader<EntityBlockCollisionEvent>,
) {
    for event in events.read() {
        commands.entity(event.entity).insert(Despawned);
    }
}

fn on_entity_entity_collision(
    mut commands: Commands,
    mut players: Query<(&mut Client, &Position)>,
    target: Query<&TargetMarker>,
    mut events: EventReader<EntityEntityCollisionEvent>,
) {
    for event in events.read() {
        if target.get(event.entity2).is_ok() {
            commands.entity(event.entity1).insert(Despawned);

            for (mut client, pos) in players.iter_mut() {
                client.send_chat_message("Hit!");
                client.play_sound(
                    Sound::EntityPigDeath,
                    SoundCategory::Neutral,
                    pos.0,
                    1.0,
                    1.0,
                );
            }
        }
    }
}
