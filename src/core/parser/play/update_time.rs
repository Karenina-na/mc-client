pub fn parse(pkt: Vec<u8>) -> (i64, i64) {
    // parse
    let word_age = &pkt[0..8].to_vec();
    let time_of_day = &pkt[8..16].to_vec();
    let word_age = i64::from_be_bytes(word_age.as_slice().try_into().unwrap());
    let time_of_day = i64::from_be_bytes(time_of_day.as_slice().try_into().unwrap()) % 24000;

    (word_age, time_of_day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let pkt = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x1d, 0x39, 0x4c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x32, 0xd1,
        ];
        let (word_age, time_of_day) = parse(pkt);
        assert_eq!(word_age, 1915212);
        assert_eq!(time_of_day, 13009);
        let day = word_age / 24000;
        assert_eq!(day, 79);
    }
}
