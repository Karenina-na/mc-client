use crate::parser::mapper;

pub fn parse(pkt: Vec<u8>) -> (String, bool) {
    // check len
    if pkt.len() - 1 != pkt[0] as usize {
        panic!("Invalid packet length");
    }
    // check type
    if pkt[2] != mapper::CHANGE_DIFFICULTY {
        panic!("Invalid packet type");
    }
    // parse
    let data = &pkt[3];
    let lock: bool = pkt[4] == 0x01;

    match data {
        0x00 => return ("peaceful".to_string(), lock),
        0x01 => return ("easy".to_string(), lock),
        0x02 => return ("normal".to_string(), lock),
        0x03 => return ("hard".to_string(), lock),
        _ => panic!("Invalid difficulty")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        // 050c000000
        let pkt = vec![
            0x04, 0x00, 0x0c, 0x01, 0x00
        ];
        let (difficulty, lock) = parse(pkt);
        assert_eq!(difficulty, "easy");
        assert_eq!(lock, false);
    }
}