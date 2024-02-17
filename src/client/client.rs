use crate::client::msg;
use crate::client::msg::login::{handshake, login_plugin_response, login_start};
use crate::client::msg::play::confirm_tp;
use crate::client::parser;
use crate::client::parser::login::{login_plugin_request, login_success, set_compression};
use crate::client::parser::mapper;
use crate::client::parser::play::{change_difficulty, server_data, sync_player_position};
use crate::itti::basis::ITTI;
use log::{debug, error, info, warn};
use tokio::sync::mpsc::Receiver;

enum Status {
    HANDSHAKE,
    LOGIN,
    PLAY,
}

pub struct Client {
    username: String,
    protocol_version: i32,
    uuid: Option<Vec<u8>>,
    threshold: Option<i32>,
    difficulty: Option<String>,
    motor: Option<String>,
    icon: Option<Vec<u8>>,
    enforce_chat: Option<bool>,
    position: Option<(f64, f64, f64, f32, f32)>,

    compress: bool,

    status: Status,
}

impl Client {
    pub fn new(username: String, protocol_version: i32) -> Client {
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
            compress: false,
            status: Status::HANDSHAKE,
        }
    }
    pub async fn start(&mut self, itti: &mut ITTI, receiver: &mut Receiver<Vec<u8>>) -> () {
        // start itti
        match itti.build().await {
            Ok(_) => {
                debug!("Built ITTI");
            }
            Err(e) => {
                error!("Failed to build ITTI: {}", e.to_string());
            }
        }

        // Send handshake
        let handshake = handshake::new(
            self.protocol_version,
            itti.ip.clone(),
            itti.port.parse::<u16>().unwrap(),
            true,
        );
        match itti.send(handshake).await {
            Ok(_) => {
                debug!("Sent handshake");
            }
            Err(e) => {
                error!("Failed to send handshake: {}", e.to_string());
            }
        }

        // Send login start
        let login_start = login_start::new(self.username.clone());
        match itti.send(login_start).await {
            Ok(_) => {
                debug!("Sent login start");
            }
            Err(e) => {
                error!("Failed to send login start: {}", e.to_string());
            }
        }

        // Start listening
        loop {
            tokio::select! {
                Some(packet) = receiver.recv() => { // console
                    if packet == b"/quit" {
                        info!("client quit");
                        break;
                    }
                    match itti.send(packet).await {
                        Ok(_) => {
                            debug!("Sent packet successfully");
                        }
                        Err(e) => {
                            error!("Failed to send packet: {}", e.to_string());
                        }
                    }
                },
                Ok(packet) = itti.recv() => { // server
                    if packet.len() == 0 {
                        info!("Server closed");
                        break;
                    }
                    self.handle_packet(packet, itti).await;
                }
            }
        }
    }

    pub async fn handle_packet(&mut self, packet: Vec<u8>, itti: &ITTI) {
        match self.threshold {
            Some(threshold) => {
                if packet.len() < threshold as usize || packet[0] as usize != packet.len() {
                    // skip compression or invalid packet
                    return;
                }
            }
            None => {}
        }

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
                info!("Set compression: {}", self.threshold.as_ref().unwrap());
                // check compress
                match self.threshold {
                    Some(threshold) => {
                        if threshold >= 0 {
                            self.compress = true
                        }
                    }
                    None => {}
                }
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
                info!(
                    "Logged in: {}, uuid: {:?}",
                    username,
                    self.uuid
                        .as_ref()
                        .iter()
                        .map(|x| format!("0x{:02x?} ", x))
                        .collect::<String>()
                );
                debug!("Changing status to play");
            }
            mapper::LOGIN_PLUGIN_REQUEST => {
                // 0x04
                let (id, channel, data) = login_plugin_request::parse(packet);
                let response = login_plugin_response::new(id, false, self.compress); // no check
                match itti.send(response).await {
                    Ok(_) => {
                        debug!("Sent login plugin response");
                    }
                    Err(e) => {
                        warn!("Failed to send login plugin response: {}", e.to_string());
                    }
                }
                debug!(
                    "Login plugin request: id: {}, channel: {}, data: {:?}",
                    id, channel, data
                );
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
                info!(
                    "Difficulty: {}, lock: {}",
                    self.difficulty.as_ref().unwrap(),
                    lock
                );
            }
            mapper::KEEP_LIVE => {
                // 0x23
                let id = parser::play::keep_live::parse(packet);
                let response = msg::play::keep_live::new(id.clone(), self.compress);
                match itti.send(response).await {
                    Ok(_) => {
                        debug!("Sent keep live response");
                    }
                    Err(e) => {
                        warn!("Failed to send keep live response: {}", e.to_string());
                    }
                }
                debug!("Keep live: {:?}", id);
            }
            mapper::SERVER_DATA => {
                // 0x45
                let (moto, icon, enforce_chat) = server_data::parse(packet);
                self.motor = Some(moto);
                self.icon = Some(icon);
                self.enforce_chat = Some(enforce_chat);
                info!(
                    "Server data: moto: {}, enforce chat: {}",
                    self.motor.as_ref().unwrap(),
                    self.enforce_chat.as_ref().unwrap()
                );
            }
            mapper::SYNC_PLAYER_POSITION => {
                // 0x3c
                let (x, y, z, yaw, pitch, is_abs, tp_id) = sync_player_position::parse(packet);
                self.position = Some((x, y, z, yaw, pitch));
                let response = confirm_tp::new(tp_id, self.compress);
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
                info!(
                    "Position: \n x: {},\n y: {},\n z: {},\n yaw: {},\n pitch: {}\n",
                    self.position.as_ref().unwrap().0,
                    self.position.as_ref().unwrap().1,
                    self.position.as_ref().unwrap().2,
                    self.position.as_ref().unwrap().3,
                    self.position.as_ref().unwrap().4
                );
            }
            _ => {}
        }
    }
}
