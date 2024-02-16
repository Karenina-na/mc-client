use crate::client::msg;
use crate::client::msg::login::login_plugin_response;
use crate::client::msg::play::confirm_tp;
use crate::client::parser;
use crate::client::parser::login::{login_plugin_request, login_success, set_compression};
use crate::client::parser::mapper;
use crate::client::parser::play::{change_difficulty, server_data, sync_player_position};
use crate::itti::basis::ITTI;
use log::{debug, info, warn};

enum Status {
    HANDSHAKE,
    LOGIN,
    PLAY,
}

struct Client {
    username: String,
    protocol_version: i32,
    uuid: Option<Vec<u8>>,
    threshold: Option<i32>,
    difficulty: Option<String>,
    motor: Option<String>,
    icon: Option<Vec<u8>>,
    enforce_chat: Option<bool>,
    position: Option<(f64, f64, f64, f32, f32)>,

    status: Status,
}

impl Client {
    fn new(username: String, protocol_version: i32) -> Client {
        Client {
            username,
            protocol_version,
            uuid: None,
            threshold: None,
            difficulty: None,
            motor: None,
            icon: None,
            enforce_chat: None,
            position: None,
            status: Status::HANDSHAKE,
        }
    }

    async fn handle_packet(&mut self, packet: Vec<u8>, itti: &ITTI) {
        match self.status {
            Status::HANDSHAKE => {
                let packet_id = packet[1];
                self.handle_handshake_packet(packet, packet_id, itti).await;
            }
            Status::LOGIN => {
                let packet_id = packet[2];
                self.handle_login_packet(packet, packet_id, itti).await;
            }
            Status::PLAY => {
                let packet_id = packet[2];
                self.handle_play_packet(packet, packet_id, itti).await;
            }
        }
    }

    #[allow(unused_variables)]
    async fn handle_handshake_packet(&mut self, packet: Vec<u8>, packet_id: u8, itti: &ITTI) {
        match packet_id {
            mapper::SET_COMPRESSION => {
                // 0x03
                let threshold = set_compression::parse(packet);
                self.threshold = Some(threshold);
                self.status = Status::LOGIN;
            }
            n => {
                info!("Unknown packet id: {}", n);
            }
        }
    }

    #[allow(unused_variables)]
    async fn handle_login_packet(&mut self, packet: Vec<u8>, packet_id: u8, itti: &ITTI) {
        match packet_id {
            mapper::LOGIN_SUCCESS => {
                // 0x02
                let (uuid, username) = login_success::parse(packet);
                self.uuid = Some(uuid);
                if username != self.username {
                    warn!(
                        "Server returned a different username: {}, expected: {}",
                        username, self.username
                    );
                }
                self.status = Status::PLAY;
            }
            mapper::LOGIN_PLUGIN_REQUEST => {
                // 0x04
                let (id, channel, data) = login_plugin_request::parse(packet);
                let response = login_plugin_response::new(id, false);
                // itti.send(response).await;
                match itti.send(response).await {
                    Ok(_) => {
                        debug!("Sent login plugin response");
                    }
                    Err(e) => {
                        warn!("Failed to send login plugin response: {}", e.to_string());
                    }
                }
            }
            n => {
                info!("Unknown packet id: {}", n);
            }
        }
    }

    #[allow(unused_variables)]
    async fn handle_play_packet(&mut self, packet: Vec<u8>, packet_id: u8, itti: &ITTI) {
        match packet_id {
            mapper::CHANGE_DIFFICULTY => {
                // 0x0d
                let (difficulty, lock) = change_difficulty::parse(packet);
                self.difficulty = Some(difficulty);
            }
            mapper::KEEP_LIVE => {
                // 0x23
                let id = parser::play::keep_live::parse(packet);
                let response = msg::play::keep_live::new(id);
                match itti.send(response).await {
                    Ok(_) => {
                        debug!("Sent keep live response");
                    }
                    Err(e) => {
                        warn!("Failed to send keep live response: {}", e.to_string());
                    }
                }
            }
            mapper::SERVER_DATA => {
                // 0x45
                let (moto, icon, enforce_chat) = server_data::parse(packet);
                self.motor = Some(moto);
                self.icon = Some(icon);
                self.enforce_chat = Some(enforce_chat);
            }
            mapper::SYNC_PLAYER_POSITION => {
                // 0x3c
                let (x, y, z, yaw, pitch, is_abs, tp_id) = sync_player_position::parse(packet);
                self.position = Some((x, y, z, yaw, pitch));
                let response = confirm_tp::new(tp_id);
                match itti.send(response).await {
                    Ok(_) => {
                        debug!("Sent sync player position response");
                    }
                    Err(e) => {
                        warn!(
                            "Failed to send sync player position response: {}",
                            e.to_string()
                        );
                    }
                }
            }
            _ => {}
        }
    }
}
