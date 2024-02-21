use crate::util;

pub fn parse(pkt: Vec<u8>) -> (f32, i32, i32) {
    // parse
    let exp_bar = f32::from_be_bytes(pkt[0..4].try_into().unwrap());

    let num = util::split::get_var_int_num(pkt[4..].to_vec(), 2);
    let level = util::transfer_var::var_int2uint(pkt[4..4 + num[0]].to_vec())[0];
    let exp_level = util::transfer_var::var_int2uint(pkt[4 + num[0]..].to_vec())[0];

    (exp_bar, level, exp_level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let pkt = vec![0x3e, 0x30, 0x8d, 0x2f, 0x0b, 0xc8, 0x01];
        let (exp_bar, level, exp_level) = parse(pkt);
        assert_eq!(exp_bar, 0.17241357);
        assert_eq!(level, 11);
        assert_eq!(exp_level, 200);

        let pikt = vec![0x3f, 0x04, 0x69, 0xeb, 0x0b, 0xd2, 0x01];
        let (exp_bar, level, exp_level) = parse(pikt);
        assert_eq!(exp_bar, 0.5172412);
        assert_eq!(level, 11);
        assert_eq!(exp_level, 210);
    }
}
