use crate::core::msg::mapper;
use chrono::Utc;

pub fn new(msg: String, compress: bool) -> Vec<u8> {
    let mut chat_message_pkt: Vec<u8> = Vec::new();
    if compress {
        chat_message_pkt.push(0x00);
    }
    // chat
    chat_message_pkt.push(mapper::CHAT_MESSAGE);
    chat_message_pkt.push(msg.len() as u8);
    chat_message_pkt = [chat_message_pkt, msg.as_bytes().to_vec()].concat();
    // timestamp
    chat_message_pkt = [
        chat_message_pkt,
        Utc::now().timestamp().to_be_bytes().to_vec(),
    ]
    .concat();
    // salt
    for _ in 0..8 {
        chat_message_pkt.push(0x00);
    }
    // has signature
    chat_message_pkt.push(0x00);
    // acknowledge
    for _ in 0..4 {
        chat_message_pkt.push(0x00);
    }
    chat_message_pkt = [vec![chat_message_pkt.len() as u8], chat_message_pkt].concat();
    chat_message_pkt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mc_chat_message_compress() {
        let msg = "nihao".to_string();
        let chat_message_pkt = new(msg, true);
        let front = chat_message_pkt[0..9].to_vec();
        let back = chat_message_pkt[17..chat_message_pkt.len()].to_vec();
        assert_eq!(
            front,
            vec![0x1d, 0x00, 0x05, 0x05, 0x6e, 0x69, 0x68, 0x61, 0x6f]
        );
        assert_eq!(
            back,
            vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }
}
