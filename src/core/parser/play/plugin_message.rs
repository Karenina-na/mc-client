pub fn parse(pkt: Vec<u8>) -> (String, String) {
    // parse
    let channel_n = pkt[0] as usize;
    let channel = String::from_utf8(pkt[1..channel_n + 1].to_vec()).unwrap();

    if channel != "minecraft:brand" {
        return (channel, "".to_string());
    }
    let data_n = pkt[channel_n + 1] as usize;
    let data = String::from_utf8(pkt[channel_n + 2..channel_n + 2 + data_n].to_vec()).unwrap();
    (channel, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let pkt = vec![
            0x0f, 0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74, 0x3a, 0x62, 0x72, 0x61,
            0x6e, 0x64, 0x06, 0x53, 0x70, 0x69, 0x67, 0x6f, 0x74,
        ];
        let (channel, data) = parse(pkt);
        assert_eq!(channel, "minecraft:brand");
        assert_eq!(data, "Spigot");
    }
}
