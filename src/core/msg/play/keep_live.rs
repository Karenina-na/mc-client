use crate::core::msg::mapper;

pub fn new(id: Vec<u8>, compress: bool) -> Vec<u8> {
    let mut keep_alive_pkt: Vec<u8> = Vec::new();
    if compress {
        keep_alive_pkt.push(0x00);
    }
    keep_alive_pkt.push(mapper::KEEP_LIVE);
    keep_alive_pkt.extend(id);
    keep_alive_pkt = [vec![keep_alive_pkt.len() as u8], keep_alive_pkt].concat();
    keep_alive_pkt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mc_keep_alive_compress() {
        let id: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x07, 0x1B, 0x44, 0xF3];
        let result = new(id, true);
        //0a001200000000071b44f3
        let expected: Vec<u8> = vec![
            0x0A, 0x00, 0x12, 0x00, 0x00, 0x00, 0x00, 0x07, 0x1B, 0x44, 0xF3,
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mc_keep_alive_no_compress() {
        let id: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x07, 0x1B, 0x44, 0xF3];
        let result = new(id, false);
        //0a001200000000071b44f3
        let expected: Vec<u8> = vec![0x09, 0x12, 0x00, 0x00, 0x00, 0x00, 0x07, 0x1B, 0x44, 0xF3];
        assert_eq!(result, expected);
    }
}
