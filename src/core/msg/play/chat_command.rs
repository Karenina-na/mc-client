use crate::core::msg::mapper;
use chrono::Utc;

pub fn new(command: String, compress: bool) -> Vec<u8> {
    let mut chat_command_pkt: Vec<u8> = Vec::new();
    if compress {
        chat_command_pkt.push(0x00);
    }
    // chat
    chat_command_pkt.push(mapper::CHAT_COMMAND);
    chat_command_pkt.push(command.len() as u8);
    chat_command_pkt = [chat_command_pkt, command.as_bytes().to_vec()].concat();
    // timestamp
    chat_command_pkt = [
        chat_command_pkt,
        Utc::now().timestamp().to_be_bytes().to_vec(),
    ]
    .concat();
    // salt
    for _ in 0..8 {
        chat_command_pkt.push(0x00);
    }
    // array length
    chat_command_pkt.push(0x00);
    // acknowledge
    for _ in 0..4 {
        chat_command_pkt.push(0x00);
    }
    chat_command_pkt = [vec![chat_command_pkt.len() as u8], chat_command_pkt].concat();
    chat_command_pkt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mc_chat_command_compress() {
        let command = "ping".to_string();
        let pkt = new(command, false);
        let front = pkt[0..7].to_vec();
        let back = pkt[15..pkt.len()].to_vec();
        assert_eq!(front, vec![0x1b, 0x04, 0x04, 0x70, 0x69, 0x6e, 0x67]);
        assert_eq!(
            back,
            vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }
}
