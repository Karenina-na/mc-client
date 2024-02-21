use crate::core::msg::mapper;

pub fn new(
    locate: String,
    view_distance: u8,
    chat_mod: u8,
    enable_chat_color: bool,
    compress: bool,
) -> Vec<u8> {
    let mut pclient_information_pkt: Vec<u8> = Vec::new();
    if compress {
        pclient_information_pkt.push(0x00);
    }
    pclient_information_pkt.push(mapper::CLIENT_INFORMATION);
    pclient_information_pkt.push(locate.len() as u8);
    pclient_information_pkt.extend(locate.as_bytes());
    pclient_information_pkt.push(view_distance);
    pclient_information_pkt.push(chat_mod);
    pclient_information_pkt.push(if enable_chat_color { 0x01 } else { 0x00 });
    pclient_information_pkt.push(0x41);
    pclient_information_pkt.push(0x00);
    pclient_information_pkt.push(0x00);
    pclient_information_pkt.push(0x01);
    pclient_information_pkt = [
        vec![pclient_information_pkt.len() as u8],
        pclient_information_pkt,
    ]
    .concat();

    pclient_information_pkt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_information_compressed() {
        let locate = String::from("en_US");
        let view_distance = 8;
        let chat_mod = 0;
        let enable_chat_color = true;
        let compress = false;
        let pkt = new(locate, view_distance, chat_mod, enable_chat_color, compress);
        assert_eq!(
            pkt,
            vec![
                0x0e, 0x08, 0x05, 0x65, 0x6e, 0x5f, 0x55, 0x53, 0x08, 0x00, 0x01, 0x41, 0x00, 0x00,
                0x01
            ]
        );
    }

    #[test]
    fn test_client_information_no_compress() {
        let locate = String::from("en_US");
        let view_distance = 8;
        let chat_mod = 0;
        let enable_chat_color = true;
        let compress = true;
        let pkt = new(locate, view_distance, chat_mod, enable_chat_color, compress);
        assert_eq!(
            pkt,
            vec![
                0x0f, 0x00, 0x08, 0x05, 0x65, 0x6e, 0x5f, 0x55, 0x53, 0x08, 0x00, 0x01, 0x41, 0x00,
                0x00, 0x01
            ]
        );
    }
}
