use crate::core::msg;
use crate::core::msg::login::{handshake, login_plugin_response, login_start};
use crate::core::msg::play::confirm_tp;
use crate::core::parser;
use crate::core::parser::login::{login_plugin_request, login_success, set_compression};
use crate::core::parser::mapper;
use crate::core::parser::play::{change_difficulty, server_data, sync_player_position};
use crate::itti::basis::ITTI;
use crate::util;
use console::style;
use log::{debug, error, info, warn};
use msg::play::{chat_command, chat_message, plugin_message, respawn};
use tokio::sync::mpsc::{Receiver, Sender};

enum Status {
    HANDSHAKE,
    LOGIN,
    PLAY,
}

pub struct Client {
    // tcp packet
    buffer: Option<Vec<u8>>,
    val: i32,

    // player
    username: String,
    protocol_version: i32,
    uuid: Option<Vec<u8>>,
    exp_bar: Option<f32>,
    level: Option<i32>,
    exp_level: Option<i32>,
    health: Option<f32>,
    food: Option<i32>,
    saturation: Option<f32>,

    // server
    threshold: Option<i32>,
    compress: bool,
    difficulty: Option<String>,
    motor: Option<String>,
    icon: Option<Vec<u8>>,
    enforce_chat: Option<bool>,
    lang: String,

    // position
    position: Option<(f64, f64, f64, f32, f32)>,

    // time
    time: Option<(i64, i64, i64)>,
    tps: Option<f32>,

    // status
    status: Status,
}

//  base
impl Client {
    pub fn new(username: String, protocol_version: i32, lang: String) -> Client {
        Client {
            buffer: None,
            val: 0,
            username,
            protocol_version,
            uuid: None,
            exp_bar: None,
            health: None,
            food: None,
            saturation: None,
            level: None,
            exp_level: None,
            threshold: None,
            difficulty: None,
            motor: None,
            icon: None,
            lang,
            enforce_chat: None,
            position: None,
            compress: false,
            time: None,
            tps: None,
            status: Status::HANDSHAKE,
        }
    }
    pub async fn start(
        &mut self,
        itti: &mut ITTI,
        command_rx: &mut Receiver<Vec<String>>,
        response_tx: &Sender<Vec<String>>,
        msg_tx: &Sender<Vec<String>>,
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
        let login_start = login_start::new(self.username.clone(), vec![]);
        match itti.send(login_start).await {
            Ok(_) => {
                debug!("Sent login start");
            }
            Err(e) => {
                error!("Failed to send login start: {}", e.to_string());
            }
        }

        // Start listening
        self.start_listen(itti, command_rx, response_tx, msg_tx)
            .await;
    }

    pub fn reset(&mut self) {
        self.buffer = None;
        self.val = 0;
        self.uuid = None;
        self.threshold = None;
        self.difficulty = None;
        self.motor = None;
        self.icon = None;
        self.enforce_chat = None;
        self.position = None;
        self.compress = false;
        self.status = Status::HANDSHAKE;
        self.time = None;
        self.tps = None;
        self.exp_bar = None;
        self.level = None;
        self.exp_level = None;
        self.health = None;
        self.food = None;
        self.saturation = None;
    }

    async fn start_listen(
        &mut self,
        itti: &mut ITTI,
        command_rx: &mut Receiver<Vec<String>>,
        response_tx: &Sender<Vec<String>>,
        msg_tx: &Sender<Vec<String>>,
    ) {
        loop {
            tokio::select! {
                // console
                Some(packet) = command_rx.recv() => {
                    if packet.len() == 0 {
                        info!("client quit");
                        break;
                    }
                    // process command
                    self.handle_command(packet, itti, response_tx).await;
                },

                // server
                Ok(mut packet) = itti.recv() => {
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
                        self.handle_packet(buffer, itti, msg_tx).await;

                    }else if self.buffer.is_some() {
                        // not enough length for var int
                        let mut buffer = self.buffer.clone().unwrap();
                        buffer.extend(packet.clone());
                        self.buffer = None;
                        packet = buffer;
                    }

                    // more packet
                    let (mut packets, val, res) = util::split::split_tcp_packet(packet);
                    if val != 0 {
                        self.val = val;
                        self.buffer = Some(packets.pop().unwrap());

                    }else if res.len() != 0 {
                        // handle not enough length for var int
                        self.buffer = Some(res);
                    }

                    // handle packets
                    for p in packets {
                        self.handle_packet(p, itti, msg_tx).await;
                    }
                }
            }
        }
    }
}

//  handle packet
impl Client {
    pub async fn handle_packet(
        &mut self,
        packet: Vec<u8>,
        itti: &ITTI,
        msg_tx: &Sender<Vec<String>>,
    ) {
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
                    let packet = match util::zlib::decompress(packet) {
                        Ok(p) => p,
                        Err(e) => {
                            warn!("Failed to decompress: {}", e.to_string());
                            return;
                        }
                    };
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
                    self.handle_play_packet(packet, packet_id as u8, itti, msg_tx)
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
                        self.handle_play_packet(packet, packet_id, itti, msg_tx)
                            .await;
                        return;
                    }

                    // len > threshold (compressed)
                    let packet = match util::zlib::decompress(packet) {
                        Ok(p) => p,
                        Err(e) => {
                            warn!("Failed to decompress: {}", e.to_string());
                            return;
                        }
                    };
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
                    self.handle_play_packet(packet, packet_id, itti, msg_tx)
                        .await;
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
                info!(
                    "Login plugin request: id: {}, channel: {}, data: {:?}",
                    id, channel, data
                );
            }
            _ => {}
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
                info!(
                    "Login plugin request: id: {}, channel: {}, data: {:?}",
                    id, channel, data
                );
            }
            _ => {}
        }
    }

    #[allow(unused_variables)]
    async fn handle_play_packet(
        &mut self,
        packet: Vec<u8>,
        packet_id: u8,
        itti: &ITTI,
        msg_tx: &Sender<Vec<String>>,
    ) {
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
                match msg_tx
                    .send(vec![format!(
                        "Server difficulty: {}, is locked: {}",
                        style(self.difficulty.as_ref().unwrap()).green(),
                        style(lock).green()
                    )])
                    .await
                {
                    Ok(_) => {
                        debug!("Sent difficulty");
                    }
                    Err(e) => {
                        warn!("Failed to send difficulty: {}", e.to_string());
                    }
                }
            }
            mapper::KEEP_LIVE => {
                // 0x23
                let id = parser::play::keep_live::parse(packet);
                debug!("Keep live: {:?}", id);
                let response = msg::play::keep_live::new(id.clone(), self.compress);
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
                let (mut moto, icon, enforce_chat) = server_data::parse(packet);
                moto = moto.replace("{\"text\":\"", "").replace("\"}", "");
                self.motor = Some(moto);
                self.icon = Some(icon);
                self.enforce_chat = Some(enforce_chat);
                info!(
                    "Server data: moto: {}, enforce chat: {}",
                    self.motor.as_ref().unwrap(),
                    self.enforce_chat.as_ref().unwrap()
                );
                match msg_tx
                    .send(vec![format!(
                        "moto: {}",
                        style(self.motor.as_ref().unwrap()).white()
                    )])
                    .await
                {
                    Ok(_) => {
                        debug!("Sent server data");
                    }
                    Err(e) => {
                        warn!("Failed to send server data: {}", e.to_string());
                    }
                }
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
            mapper::PLUGIN_MESSAGE => {
                // 0x17
                let (channel, data) = parser::play::plugin_message::parse(packet);
                info!("Plugin message: channel- {}, data- {:?}", channel, data);
                match channel.as_str() {
                    "minecraft:brand" => {
                        // send brand
                        let response = plugin_message::new(
                            "minecraft:brand".to_string(),
                            "Minecraft-Console-Client/1.20.2".to_string(),
                            self.compress,
                        );
                        match itti.send(response).await {
                            Ok(_) => {
                                debug!("Sent plugin message response");
                            }
                            Err(e) => {
                                warn!("Failed to send plugin message response: {}", e.to_string());
                            }
                        }
                        // send client information
                        let response = msg::play::client_information::new(
                            self.lang.clone(),
                            8,
                            0,
                            true,
                            self.compress,
                        );
                        match itti.send(response).await {
                            Ok(_) => {
                                debug!("Sent client information");
                            }
                            Err(e) => {
                                warn!("Failed to send client information: {}", e.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            mapper::DISCONNECT => {
                // 0x1a
                let reason = parser::play::disconnect::parse(packet);
                info!("Disconnect: {}", reason);
            }
            mapper::SYSTEM_CHAT_MESSAGE => {
                // 0x64
                let (data, is_overlay) = parser::play::system_chat_message::parse(packet);
                info!("System chat message: {}, overlay: {}", data, is_overlay);
                match msg_tx.send(vec!["system chat".to_string(), data]).await {
                    Ok(_) => {
                        debug!("Sent system chat message");
                    }
                    Err(e) => {
                        warn!("Failed to send system chat message: {}", e.to_string());
                    }
                }
            }
            mapper::DISGUISED_CHAT_MESSAGE => {
                // 0x1b
                let (msg, chat_type, chat_type_name, has_target_name, target_name) =
                    parser::play::disguised_chat_message::parse(packet);
                info!(
                    "Disguised chat message: msg: {}, chat type: {}, chat type name: {}, has target name: {}, target name: {}",
                    msg, chat_type, chat_type_name, has_target_name, target_name
                );
                match msg_tx.send(vec!["disguised chat".to_string(), msg]).await {
                    Ok(_) => {
                        debug!("Sent disguised chat message");
                    }
                    Err(e) => {
                        warn!("Failed to send disguised chat message: {}", e.to_string());
                    }
                }
            }
            mapper::UPDATE_TIME => {
                // 0x5e
                let (word_age, time_of_day) = parser::play::update_time::parse(packet);
                let day = word_age / 24000;
                // cal tps
                match self.time {
                    Some((last_word_age, last_time_of_day, last_day)) => {
                        // 1 tick = 1/20 sec
                        let tps = (word_age - last_word_age) as f32;
                        match self.tps {
                            Some(last_tps) => {
                                self.tps = Some((last_tps + tps) / 2.0);
                            }
                            None => {
                                self.tps = Some(tps);
                            }
                        }
                    }
                    None => {}
                }
                match self.tps {
                    Some(tps) => {
                        debug!(
                            "Update time: word age: {}, day: {}, time of day: {}, TPS: {:.2}",
                            word_age, day, time_of_day, tps
                        );
                    }
                    None => {
                        debug!(
                            "Update time: word age: {}, day: {}, time of day: {}, TPS: None",
                            word_age, day, time_of_day
                        );
                    }
                }
                self.time = Some((word_age, time_of_day, day));
            }
            mapper::SET_EXPERIENCE => {
                // 0x56
                let (exp_bar, level, exp_level) = parser::play::set_experience::parse(packet);
                self.exp_bar = Some(exp_bar);
                self.level = Some(level);
                self.exp_level = Some(exp_level);
                info!(
                    "Set experience: exp bar: {}, level: {}, exp (level): {}",
                    self.exp_bar.as_ref().unwrap(),
                    self.level.as_ref().unwrap(),
                    self.exp_level.as_ref().unwrap()
                );
            }
            mapper::SET_HEALTH => {
                // 0x57
                let (health, food, saturation) = parser::play::set_health::parse(packet);
                info!(
                    "Set health: health: {}, food: {}, saturation: {}",
                    health, food, saturation
                );
                self.health = Some(health);
                self.food = Some(food);
                self.saturation = Some(saturation);
            }
            _ => {}
        }
    }
}

//  command response
impl Client {
    async fn handle_command(
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
            "position" => match response_tx.send(vec![self.get_position()]).await {
                Ok(_) => {
                    debug!("Sent position");
                }
                Err(e) => {
                    error!("Failed to send position: {}", e.to_string());
                }
            },
            "server" => match response_tx.send(vec![self.get_server_data()]).await {
                Ok(_) => {
                    debug!("Sent server data");
                }
                Err(e) => {
                    error!("Failed to send server data: {}", e.to_string());
                }
            },
            "chat" => {
                let response = self.chat_message(packet[1].clone());
                match itti.send(response).await {
                    Ok(_) => {
                        debug!("Sent chat message: {}", packet[1]);
                    }
                    Err(e) => {
                        error!("Failed to send chat message: {}", e.to_string());
                    }
                }
            }
            "command" => {
                let response = self.chat_command(packet[1].clone());
                match itti.send(response).await {
                    Ok(_) => {
                        debug!("Sent chat command: {}", packet[1]);
                    }
                    Err(e) => {
                        error!("Failed to send chat command: {}", e.to_string());
                    }
                }
            }
            "time" => match response_tx.send(vec![self.get_time()]).await {
                Ok(_) => {
                    debug!("Sent time");
                }
                Err(e) => {
                    error!("Failed to send time: {}", e.to_string());
                }
            },
            "tps" => match response_tx.send(vec![self.get_tps()]).await {
                Ok(_) => {
                    debug!("Sent tps");
                }
                Err(e) => {
                    error!("Failed to send tps: {}", e.to_string());
                }
            },
            "exp" => match response_tx.send(vec![self.get_exp()]).await {
                Ok(_) => {
                    debug!("Sent exp");
                }
                Err(e) => {
                    error!("Failed to send exp: {}", e.to_string());
                }
            },
            "health" => match response_tx.send(vec![self.get_health()]).await {
                Ok(_) => {
                    debug!("Sent health");
                }
                Err(e) => {
                    error!("Failed to send health: {}", e.to_string());
                }
            },
            _ => {
                warn!("Unknown command: {}", packet[0]);
            }
        }
    }
    #[allow(unused_variables)]
    #[allow(unused)]
    pub fn get_position(&self) -> String {
        match self.position {
            Some((x, y, z, yaw, pitch)) => {
                format!(
                    "x: {}, y: {}, z: {}, yaw: {}, pitch: {}",
                    style(x).green(),
                    style(y).green(),
                    style(z).green(),
                    style(yaw).cyan(),
                    style(pitch).cyan()
                )
            }
            _ => style("No position").red().to_string(),
        }
    }

    #[allow(unused_variables)]
    #[allow(unused)]
    pub fn get_server_data(&self) -> String {
        match (self.motor.clone(), self.icon.clone(), self.enforce_chat) {
            (Some(motor), Some(icon), Some(enforce_chat)) => {
                format!(
                    "motor: {}, enforce chat: {}",
                    style(motor).white(),
                    style(enforce_chat).red()
                )
            }
            _ => style("No server data").red().to_string(),
        }
    }

    #[allow(unused_variables)]
    #[allow(unused)]
    pub fn respawn(&self) -> Vec<u8> {
        respawn::new(self.compress)
    }

    #[allow(unused_variables)]
    #[allow(unused)]
    pub fn chat_message(&self, msg: String) -> Vec<u8> {
        chat_message::new(msg, self.compress)
    }

    #[allow(unused_variables)]
    #[allow(unused)]
    pub fn chat_command(&self, command: String) -> Vec<u8> {
        chat_command::new(command, self.compress)
    }

    #[allow(unused_variables)]
    #[allow(unused)]
    pub fn get_time(&self) -> String {
        match self.time {
            Some((word_age, time_of_day, day)) => {
                format!(
                    "word age: {}, day: {}, time of this day: {}",
                    style(word_age).white(),
                    style(day).cyan(),
                    style(time_of_day).green()
                )
            }
            _ => style("No time").red().to_string(),
        }
    }

    #[allow(unused_variables)]
    #[allow(unused)]
    pub fn get_tps(&self) -> String {
        match self.tps {
            Some(tps) => format!("TPS: {:.2}", style(tps).green()),
            _ => style("No TPS").red().to_string(),
        }
    }

    #[allow(unused_variables)]
    #[allow(unused)]
    pub fn get_exp(&self) -> String {
        match (self.exp_bar, self.level, self.exp_level) {
            (Some(exp_bar), Some(level), Some(exp_level)) => {
                format!(
                    "exp bar: {:.2}, level: {}, exp: {}",
                    style(exp_bar).white(),
                    style(level).green(),
                    style(exp_level).white()
                )
            }
            _ => style("No exp").red().to_string(),
        }
    }

    #[allow(unused_variables)]
    #[allow(unused)]
    pub fn get_health(&self) -> String {
        match (self.health, self.food, self.saturation) {
            (Some(health), Some(food), Some(saturation)) => {
                format!(
                    "health: {}, food: {}, saturation: {}",
                    style(health).red(),
                    style(food).green(),
                    style(saturation).yellow()
                )
            }
            _ => style("No health").red().to_string(),
        }
    }
}
