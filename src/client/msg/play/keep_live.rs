pub fn new(id: Vec<u8>) -> Vec<u8> {
    let mut keep_alive_pkt: Vec<u8> = Vec::new();
    keep_alive_pkt.push(0x00);
    keep_alive_pkt.push(0x12);
    keep_alive_pkt.extend(id);
    keep_alive_pkt = [vec![keep_alive_pkt.len() as u8], keep_alive_pkt].concat();
    keep_alive_pkt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mc_keep_alive() {
        let id: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x07, 0x1B, 0x44, 0xF3];
        let result = new(id);
        //0a001200000000071b44f3
        let expected: Vec<u8> = vec![
            0x0A, 0x00, 0x12, 0x00, 0x00, 0x00, 0x00, 0x07, 0x1B, 0x44, 0xF3,
        ];
        assert_eq!(result, expected);
    }
}
