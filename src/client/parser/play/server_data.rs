use crate::client::parser::mapper;

pub fn parse(pkt: Vec<u8>) -> (String, Vec<u8>, bool) {
    // check len
    if pkt.len() - 1 != pkt[0] as usize {
        panic!("Invalid packet length");
    }
    // check type
    if pkt[2] != mapper::SERVER_DATA {
        panic!("Invalid packet type");
    }
    // parse
    let moto_n = pkt[3] as usize;
    let moto = String::from_utf8(pkt[4..4 + moto_n].to_vec()).unwrap();
    let icon_n = pkt[4 + moto_n] as usize;
    let icon = String::from_utf8(pkt[5 + moto_n..5 + moto_n + icon_n].to_vec()).unwrap();
    let enforce_chat = pkt[5 + moto_n + icon_n] == 0x01;

    return (moto, icon.into_bytes(), enforce_chat);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let pkt = vec![
            0x22, 0x00, 0x45, 0x1d, 0x7b, 0x22, 0x74, 0x65, 0x78, 0x74, 0x22, 0x3a, 0x22, 0x41,
            0x20, 0x4d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74, 0x20, 0x53, 0x65, 0x72,
            0x76, 0x65, 0x72, 0x22, 0x7d, 0x00, 0x01,
        ];
        let (moto, icon, enforce_chat) = parse(pkt);
        assert_eq!(moto, "{\"text\":\"A Minecraft Server\"}");
        assert_eq!(icon, "".as_bytes());
        assert_eq!(enforce_chat, true);
    }
}
