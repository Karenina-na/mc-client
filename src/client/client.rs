use crate::client::msg;
use crate::client::msg::login::{handshake, login_plugin_response, login_start};
use crate::client::msg::play::confirm_tp;
use crate::client::parser;
use crate::client::parser::login::{login_plugin_request, login_success, set_compression};
use crate::client::parser::mapper;
use crate::client::parser::play::{change_difficulty, server_data, sync_player_position};
use crate::itti::basis::ITTI;
use crate::util;
use log::{debug, error, info, warn};
use tokio::sync::mpsc::{Receiver, Sender};

enum Status {
    HANDSHAKE,
    LOGIN,
    PLAY,
}

pub struct Client {
    buffer: Option<Vec<u8>>,
    val: i32,

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

//  base
impl Client {
    pub fn new(username: String, protocol_version: i32) -> Client {
        Client {
            buffer: None,
            val: 0,
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
    pub async fn start(
        &mut self,
        itti: &mut ITTI,
        command_rx: &mut Receiver<Vec<String>>,
        response_tx: &Sender<Vec<String>>,
    ) -> () {
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
                Some(packet) = command_rx.recv() => { // console
                    if packet.len() == 0 {
                        info!("client quit");
                        break;
                    }
                    // process command
                    self.process_command(packet, itti, response_tx).await;
                },

                Ok(mut packet) = itti.recv() => { // server
                    if packet.len() == 0 {
                        info!("Server closed");
                        break;
                    }
                    // if last pkt is not complete
                    if self.val != 0 {
                        if self.val > packet.len() as i32 { // more packet
                            self.val -= packet.len() as i32;
                            let mut buffer = self.buffer.clone().unwrap();
                            buffer.extend(packet);
                            self.buffer = Some(buffer);
                            continue;
                        }
                        // last packet
                        let mut buffer = self.buffer.clone().unwrap();
                        buffer.extend(packet[..self.val as usize].to_vec());
                        packet = packet[self.val as usize..].to_vec();
                        self.buffer = None;
                        self.val = 0;
                        self.handle_packet(buffer, itti).await;
                    }
                    // more packet
                    let (mut packets, val) = util::split::split_tcp_packet(packet);
                    if val != 0 {
                        self.val = val;
                        self.buffer = Some(packets.pop().unwrap());
                    }
                    // handle packets
                    for p in packets {
                        self.handle_packet(p, itti).await;
                    }
                }
            }
        }
    }
}

//  handle packet
impl Client {
    pub async fn handle_packet(&mut self, packet: Vec<u8>, itti: &ITTI) {
        match self.status {
            Status::HANDSHAKE => {
                // check len
                if packet.len() - 1 != packet[0] as usize {
                    warn!(
                        "Packet length mismatch: expected: {}, actual: {}",
                        packet[0],
                        packet.len() - 1
                    );
                    return;
                }
                let packet_id = packet[1];
                let packet = packet[2..].to_vec();
                self.handle_handshake_packet(packet, packet_id, itti).await;
            }
            Status::LOGIN => {
                let (packet_len, data_len, packet_id, packet) =
                    util::split::split_packet(packet, self.threshold.unwrap_or(-1));

                if data_len == -1 {
                    // no compress
                    // check len
                    if packet_len != (packet.len() + 1) as i32 {
                        warn!(
                            "Packet {} length mismatch: expected: {}, actual: {}",
                            packet_id,
                            packet_len,
                            packet.len() + 1
                        );
                        return;
                    }
                    self.handle_login_packet(packet, packet_id as u8, itti)
                        .await;
                } else if packet_id == -1 {
                    // compress
                    // check len
                    let data_len_num =
                        util::transfer_var::uint2var_int(Vec::from([data_len])).len();
                    if packet_len != (data_len_num + packet.len()) as i32 {
                        warn!(
                            "Packet(compress) length mismatch: expected: {}, actual: {}",
                            packet_len,
                            data_len_num + packet.len()
                        );
                        return;
                    }
                    // uncompressed
                    if data_len == 0 {
                        // len < threshold
                        let packet_id = packet[0];
                        let packet = packet[1..].to_vec();
                        self.handle_login_packet(packet, packet_id, itti).await;
                        return;
                    }

                    // len > threshold (compressed)
                    let packet = util::zlib::decompress(packet);
                    // check data len
                    if data_len as usize != packet.len() {
                        warn!(
                            "Data(compress) length mismatch: expected: {}, actual: {}",
                            data_len,
                            packet.len()
                        );
                        return;
                    }
                    let packet_id = packet[0];
                    let packet = packet[1..].to_vec();
                    self.handle_login_packet(packet, packet_id, itti).await;
                }
            }
            Status::PLAY => {
                let (packet_len, data_len, packet_id, packet) =
                    util::split::split_packet(packet, self.threshold.unwrap_or(-1));

                if data_len == -1 {
                    // no compress
                    // check len
                    if packet_len != (packet.len() + 1) as i32 {
                        warn!(
                            "Packet {} length mismatch: expected: {}, actual: {}",
                            packet_id,
                            packet_len,
                            packet.len() + 1
                        );
                        return;
                    }
                    self.handle_play_packet(packet, packet_id as u8, itti).await;
                } else if packet_id == -1 {
                    // compress
                    // check len
                    let data_len_num =
                        util::transfer_var::uint2var_int(Vec::from([data_len])).len();
                    if packet_len != (data_len_num + packet.len()) as i32 {
                        warn!(
                            "Packet(compress) length mismatch: expected: {}, actual: {}",
                            packet_len,
                            data_len_num + packet.len()
                        );
                        return;
                    }
                    // uncompressed
                    if data_len == 0 {
                        // len < threshold
                        let packet_id = packet[0];
                        let packet = packet[1..].to_vec();
                        self.handle_play_packet(packet, packet_id, itti).await;
                        return;
                    }

                    // len > threshold (compressed)
                    let packet = util::zlib::decompress(packet);
                    // check data len
                    if data_len as usize != packet.len() {
                        warn!(
                            "Data(compress) length mismatch: expected: {}, actual: {}",
                            data_len,
                            packet.len()
                        );
                        return;
                    }
                    let packet_id = packet[0];
                    let packet = packet[1..].to_vec();
                    self.handle_play_packet(packet, packet_id, itti).await;
                }
            }
        }
    }

    #[allow(unused_variables)]
    async fn handle_handshake_packet(&mut self, packet: Vec<u8>, packet_id: u8, itti: &ITTI) {
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
            },
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

//  command response
impl Client {
    async fn process_command(
        &mut self,
        packet: Vec<String>,
        itti: &ITTI,
        response_tx: &Sender<Vec<String>>,
    ) {
        match packet[0].as_str() {
            "respawn" => {
                let response = self.respawn();
                match itti.send(response).await {
                    Ok(_) => {
                        debug!("Sent respawn");
                    }
                    Err(e) => {
                        error!("Failed to send respawn: {}", e.to_string());
                    }
                }
            }
            "getPosition" => match response_tx.send(vec![self.get_position()]).await {
                Ok(_) => {
                    debug!("Sent position");
                }
                Err(e) => {
                    error!("Failed to send position: {}", e.to_string());
                }
            },
            "getServerData" => match response_tx.send(vec![self.get_server_data()]).await {
                Ok(_) => {
                    debug!("Sent server data");
                }
                Err(e) => {
                    error!("Failed to send server data: {}", e.to_string());
                }
            },
            _ => {
                warn!("Unknown command: {}", packet[0]);
            }
        }
    }
    #[allow(unused_variables)]
    pub fn get_position(&self) -> String {
        match self.position {
            Some((x, y, z, yaw, pitch)) => {
                format!(
                    "x: {}, y: {}, z: {}, yaw: {}, pitch: {}",
                    x, y, z, yaw, pitch
                )
            }
            _ => "No position".to_string(),
        }
    }

    #[allow(unused_variables)]
    pub fn get_server_data(&self) -> String {
        match (self.motor.clone(), self.icon.clone(), self.enforce_chat) {
            (Some(motor), Some(icon), Some(enforce_chat)) => {
                format!(
                    "motor: {}, icon: {}, enforce chat: {}",
                    motor,
                    icon.len() > 0,
                    enforce_chat
                )
            }
            _ => "No server data".to_string(),
        }
    }

    #[allow(unused_variables)]
    pub fn respawn(&self) -> Vec<u8> {
        msg::play::respawn::new(self.compress)
    }
}
