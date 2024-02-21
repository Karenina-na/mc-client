use crate::util;

pub fn parse(pkt: Vec<u8>) -> (f32, i32, f32) {
    // parse
    let health = f32::from_be_bytes(pkt[0..4].try_into().unwrap());

    let num = util::split::get_var_int_num(pkt[4..].to_vec(), 1);
    let food = util::transfer_var::var_int2uint(pkt[4..4 + num[0]].to_vec())[0];
    let saturation = f32::from_be_bytes(pkt[4 + num[0]..8 + num[0]].try_into().unwrap());

    (health, food, saturation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let pkt = vec![0x41, 0xa0, 0x00, 0x00, 0x14, 0x40, 0xa0, 0x00, 0x00];
        let (health, food, saturation) = parse(pkt);
        assert_eq!(health, 20.0);
        assert_eq!(food, 20);
        assert_eq!(saturation, 5.0);
    }
}
