pub fn parse(pkt: Vec<u8>) -> Vec<u8> {
    // check len
    if pkt.len() - 1 != pkt[0] as usize {
        panic!("Invalid packet length");
    }
    // check type
    if pkt[2] != 0x23 {
        panic!("Invalid packet type");
    }
    // parse
    let data = &pkt[3..];
    return data.to_vec();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        // 0a002300000000071b44f3
        let pkt = vec![
            0x0A, 0x00, 0x23, 0x00, 0x00, 0x00, 0x00, 0x07, 0x1B, 0x44, 0xF3,
        ];
        let data = parse(pkt);
        assert_eq!(data, vec![0x00, 0x00, 0x00, 0x00, 0x07, 0x1B, 0x44, 0xF3]);
    }
}
