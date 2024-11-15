use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use bevy_ecs::{entity::EntityHashMap, query::QueryData};
use valence::{message::ChatMessageEvent, prelude::*};

/// The active chat channels that can be used by the players.
#[derive(Default, Resource)]
pub struct ChatChannels {
    /// Maps the channel id to the active channel config and to the player-channel-config of each player in the channel.
    channels: HashMap<u64, (ChatChannelConfig, EntityHashMap<PlayerChatChannelConfig>)>,
    /// Maps a player to the channels they are in, the first set is the channels with a required prefix, the second set is the channels without a required prefix.
    players_to_channels: EntityHashMap<(HashSet<u64>, HashSet<u64>)>,
}

impl ChatChannels {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new chat channel to the chat channels.
    pub fn add_channel(&mut self, channel_id: u64, config: ChatChannelConfig) {
        self.channels
            .insert(channel_id, (config, EntityHashMap::default()));
    }

    /// Add a player to a channel.
    ///
    /// # Arguments
    /// - `channel_id`: The id of the channel.
    /// - `player_entity`: The entity of the player.
    /// - `player_config`: The player's config for this channel.
    ///
    /// # Returns
    ///
    /// - `Some(())` if the player was added to the channel.
    /// - `None` if the channel does not exist.
    pub fn add_player_to_channel(
        &mut self,
        channel_id: u64,
        player_entity: Entity,
        player_config: PlayerChatChannelConfig,
    ) -> Option<()> {
        let (channel_config, channel_members) = self.channels.get_mut(&channel_id)?;
        channel_members.insert(player_entity, player_config);

        if !self.players_to_channels.contains_key(&player_entity) {
            self.players_to_channels
                .insert(player_entity, (HashSet::new(), HashSet::new()));
        }

        let (with_prefix, without_prefix) =
            self.players_to_channels.get_mut(&player_entity).unwrap();

        if channel_config.required_prefix.is_some() {
            with_prefix.insert(channel_id);
        } else {
            without_prefix.insert(channel_id);
        }

        Some(())
    }

    /// Remove a player from a channel.
    pub fn remove_player_from_channel(
        &mut self,
        channel_id: u64,
        player_entity: Entity,
    ) -> Option<()> {
        let (_, channel_members) = self.channels.get_mut(&channel_id)?;
        channel_members.remove(&player_entity);

        if let Some((with_prefix, without_prefix)) =
            self.players_to_channels.get_mut(&player_entity)
        {
            with_prefix.remove(&channel_id);
            without_prefix.remove(&channel_id);
        }

        Some(())
    }

    /// Remove a player from all channels.
    pub fn remove_player(&mut self, player_entity: Entity) {
        for (_, (_, channel_members)) in self.channels.iter_mut() {
            channel_members.remove(&player_entity);
        }

        self.players_to_channels.remove(&player_entity);
    }
}

/// A general config of a chat channel.
#[derive(Default)]
pub struct ChatChannelConfig {
    /// If the chat message should be hidden to the sender.
    pub hide_msg_for_sender: bool,
    /// The required prefix that needs to be present in order for the message to be sent.
    /// This could be used for team chats, global chats, etc.
    pub required_prefix: Option<String>,
    /// A cooldown for the chat message.
    pub chat_cooldown: Option<Duration>,
    /// The global prefix that will be applied to all messages in this channel.
    pub global_prefix: Option<String>,
}

/// A config for a player that is specific to a chat channel.
#[derive(Default, Clone)]
pub struct PlayerChatChannelConfig {
    /// The player's permission for the channel.
    pub permission: ChatChannelPermission,
    /// The player's prefix for the channel.
    pub prefix: Option<String>,
}

/// The permissions for a specific chat channel of a player.
#[derive(Default, PartialEq, Clone, Copy)]
pub enum ChatChannelPermission {
    /// Player is able to read and write in the channel.
    ReadWrite,
    /// Player is only able to read in the channel.
    #[default]
    Read,
    /// Player is only able to write in the channel.
    Write,
}

impl ChatChannelPermission {
    /// Check if the player has the permission to read in the channel.
    pub fn can_read(&self) -> bool {
        matches!(self, Self::ReadWrite | Self::Read)
    }

    /// Check if the player has the permission to write in the channel.
    pub fn can_write(&self) -> bool {
        matches!(self, Self::ReadWrite | Self::Write)
    }
}

/// A component that stores information about the player's current chat state.
/// This needs to be attached to the player in order to use the chat system.
#[derive(Default, Component)]
pub struct ChatAbility {
    /// Messages from players with that name will be ignored.
    pub muted_players: HashSet<String>, // TODO: should this be the player's UUID instead?
    /// The last time the player sent a message.
    pub last_message_time: Option<Instant>,
}

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, chat_system)
            .insert_resource(ChatChannels::default());
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ChatQuery {
    entity: Entity,
    name: &'static Username,
    chat_ability: &'static mut ChatAbility,
    client: &'static mut Client,
}

fn chat_system(
    channels: Res<ChatChannels>,
    mut clients: Query<ChatQuery>,
    mut events: EventReader<ChatMessageEvent>,
) {
    for event in events.read() {
        let chat_message = event.message.to_string();
        let Some((channels_with_prefix, channels_without_prefix)) =
            channels.players_to_channels.get(&event.client)
        else {
            continue;
        };

        let mut sent_to_prefixed_channels = false;

        let channel_prefix_len = channels_with_prefix.len();
        for (idx, channel_id) in channels_with_prefix
            .iter()
            .chain(channels_without_prefix.iter())
            .enumerate()
        {
            let is_regular_channel = idx >= channel_prefix_len;

            // When we used a prefix to sent to a channel, we dont want to send it to the regular channels.
            // TODO: Maybe this should be a config option?
            if is_regular_channel && sent_to_prefixed_channels {
                break;
            }

            let player_channel_config = channels
                .channels
                .get(channel_id)
                .unwrap()
                .1
                .get(&event.client)
                .unwrap();

            if !player_channel_config.permission.can_write() {
                continue;
            }

            let (channel_config, channel_members) = channels.channels.get(channel_id).unwrap();

            let mut message = chat_message.clone();
            if let Some(prefix) = &channel_config.required_prefix {
                if !chat_message.starts_with(prefix) {
                    continue;
                }
                message = chat_message[prefix.len()..].trim_start().to_string();

                sent_to_prefixed_channels = true;
            }

            // Chat cooldown
            {
                let Ok(mut sender) = clients.get_mut(event.client) else {
                    continue;
                };

                if let Some(cooldown) = channel_config.chat_cooldown {
                    if let Some(last_message_time) = sender.chat_ability.last_message_time {
                        if last_message_time.elapsed() < cooldown {
                            continue;
                        }
                    }
                }

                sender.chat_ability.last_message_time = Some(Instant::now());
            }

            // Apply the player's prefix and the global prefix.
            if let Some(player_prefix) = &player_channel_config.prefix {
                message = format!("{}{}", player_prefix, message);
            }

            if let Some(global_prefix) = &channel_config.global_prefix {
                message = format!("{}{}", global_prefix, message);
            }

            let sender_name = {
                let Ok(sender) = clients.get(event.client) else {
                    continue;
                };
                sender.name.to_string()
            };

            for (player_entity, player_config) in channel_members.iter() {
                let Ok(mut receiver) = clients.get_mut(*player_entity) else {
                    continue;
                };

                if !player_config.permission.can_read() {
                    continue;
                }

                if channel_config.hide_msg_for_sender && *player_entity == event.client {
                    continue;
                }

                if receiver.chat_ability.muted_players.contains(&sender_name) {
                    continue;
                }

                receiver.client.send_chat_message(&message);
            }
        }
    }
}
