pub fn parse(pkt: Vec<u8>) -> (String, bool) {
    // parse
    let data = &pkt[0];
    let lock: bool = pkt[1] == 0x01;

    match data {
        0x00 => return ("peaceful".to_string(), lock),
        0x01 => return ("easy".to_string(), lock),
        0x02 => return ("normal".to_string(), lock),
        0x03 => return ("hard".to_string(), lock),
        _ => panic!("Invalid difficulty"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        // 050c000000
        let pkt = vec![0x01, 0x00];
        let (difficulty, lock) = parse(pkt);
        assert_eq!(difficulty, "easy");
        assert_eq!(lock, false);
    }
}
