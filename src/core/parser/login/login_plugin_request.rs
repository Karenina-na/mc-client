pub fn parse(pkt: Vec<u8>) -> (u8, String, String) {
    // id and channel
    let id = pkt[0];
    let channel_n = pkt[1] as usize;
    let channel = &pkt[2..2 + channel_n]
        .iter()
        .map(|&c| c as char)
        .collect::<String>();

    // check data
    if pkt[2 + channel_n] != 0x01 {
        panic!("Invalid packet data");
    }

    // check len
    if pkt.len() == 3 + channel_n {
        return (id, channel.to_string(), "".to_string());
    }

    // data
    let data_n = pkt[3 + channel_n] as usize;
    let data = &pkt[4 + channel_n..4 + channel_n + data_n]
        .iter()
        .map(|&c| c as char)
        .collect::<String>();

    (id, channel.to_string(), data.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data() {
        // 4a0004002b6661627269632d6e6574776f726b696e672d6170692d76313a6561726c795
        // f726567697374726174696f6e0119616476656e747572653a726567697374657265645f61726773
        let pkt = vec![
            0x00, 0x2b, 0x66, 0x61, 0x62, 0x72, 0x69, 0x63, 0x2d, 0x6e, 0x65, 0x74, 0x77, 0x6f,
            0x72, 0x6b, 0x69, 0x6e, 0x67, 0x2d, 0x61, 0x70, 0x69, 0x2d, 0x76, 0x31, 0x3a, 0x65,
            0x61, 0x72, 0x6c, 0x79, 0x5f, 0x72, 0x65, 0x67, 0x69, 0x73, 0x74, 0x72, 0x61, 0x74,
            0x69, 0x6f, 0x6e, 0x01, 0x19, 0x61, 0x64, 0x76, 0x65, 0x6e, 0x74, 0x75, 0x72, 0x65,
            0x3a, 0x72, 0x65, 0x67, 0x69, 0x73, 0x74, 0x65, 0x72, 0x65, 0x64, 0x5f, 0x61, 0x72,
            0x67, 0x73,
        ];
        let (id, channel, data) = parse(pkt);
        assert_eq!(id, 0x00);
        assert_eq!(channel, "fabric-networking-api-v1:early_registration");
        assert_eq!(data, "adventure:registered_args");
    }

    #[test]
    fn test_parse_no_data() {
        // 220004011d6661627269633a637573746f6d5f696e6772656469656e745f73796e6301
        let pkt = vec![
            0x01, 0x1d, 0x66, 0x61, 0x62, 0x72, 0x69, 0x63, 0x3a, 0x63, 0x75, 0x73, 0x74, 0x6f,
            0x6d, 0x5f, 0x69, 0x6e, 0x67, 0x72, 0x65, 0x64, 0x69, 0x65, 0x6e, 0x74, 0x5f, 0x73,
            0x79, 0x6e, 0x63, 0x01,
        ];
        let (id, channel, data) = parse(pkt);
        assert_eq!(id, 0x01);
        assert_eq!(channel, "fabric:custom_ingredient_sync");
        assert_eq!(data, "");
    }
}
