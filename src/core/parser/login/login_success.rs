pub fn parse(pkt: Vec<u8>) -> (Vec<u8>, String) {
    // parse
    let uuid = &pkt[0..16];
    let n = pkt[16] as usize;
    let username = &pkt[17..17 + n]
        .iter()
        .map(|&c| c as char)
        .collect::<String>();

    return (uuid.to_vec(), username.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        // 1c0002037f5695cc3039649caf8c000e107c14084b6172656e696e6100
        let pkt = vec![
            0x03, 0x7F, 0x56, 0x95, 0xCC, 0x30, 0x39, 0x64, 0x9C, 0xAF, 0x8C, 0x00, 0x0E, 0x10,
            0x7C, 0x14, 0x08, 0x4B, 0x61, 0x72, 0x65, 0x6E, 0x69, 0x6E, 0x61, 0x00,
        ];
        let (uuid, username) = parse(pkt);
        assert_eq!(
            uuid,
            //037f5695cc3039649caf8c000e107c14
            vec![
                0x03, 0x7F, 0x56, 0x95, 0xCC, 0x30, 0x39, 0x64, 0x9C, 0xAF, 0x8C, 0x00, 0x0E, 0x10,
                0x7C, 0x14
            ]
        );
        assert_eq!(username, "Karenina");
    }
}
