// #![cfg(feature = "chat")]
use std::time::Duration;

use chat::*;
use valence::prelude::*;
const SPAWN_Y: i32 = 64;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_plugins(ChatPlugin)
        .add_systems(Update, (init_clients, despawn_disconnected_clients))
        .run();
}

fn setup(
    mut chat_channels: ResMut<ChatChannels>,
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

    chat_channels.add_channel(
        0,
        ChatChannelConfig {
            hide_msg_for_sender: false,
            required_prefix: None,
            chat_cooldown: None,
            global_prefix: None,
        },
    );

    commands.spawn(layer);
}

#[allow(clippy::type_complexity)]
fn init_clients(
    mut chat_channels: ResMut<ChatChannels>,
    mut commands: Commands,
    mut clients: Query<
        (
            Entity,
            &mut Position,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut GameMode,
            &Username,
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
        username,
    ) in &mut clients
    {
        let layer = layers.single();

        pos.0 = [0.0, f64::from(SPAWN_Y) + 1.0, 0.0].into();
        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        visible_entity_layers.0.insert(layer);
        *game_mode = GameMode::Survival;

        commands.entity(player_ent).insert(ChatAbility::default());

        // Global chat
        chat_channels.add_channel(
            0,
            ChatChannelConfig {
                hide_msg_for_sender: false,
                required_prefix: None,
                chat_cooldown: Some(Duration::from_secs_f32(0.5)),
                global_prefix: None,
            },
        );

        // Team chat
        chat_channels.add_channel(
            1,
            ChatChannelConfig {
                hide_msg_for_sender: false,
                required_prefix: Some("@t".to_string()),
                chat_cooldown: None,
                global_prefix: Some("[§cTeam§r] ".to_string()),
            },
        );

        chat_channels.add_player_to_channel(
            0,
            player_ent,
            PlayerChatChannelConfig {
                permission: ChatChannelPermission::ReadWrite,
                prefix: Some(format!("{}> ", username.0)),
            },
        );

        chat_channels.add_player_to_channel(
            1,
            player_ent,
            PlayerChatChannelConfig {
                permission: ChatChannelPermission::ReadWrite,
                prefix: Some(format!("{}> ", username.0)),
            },
        );
    }
}
