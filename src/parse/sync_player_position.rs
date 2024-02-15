use crate::util::transfer_var;

pub fn parse(pkt: Vec<u8>) -> (f64, f64, f64, f32, f32, bool, i32) {
    // check len
    if pkt.len() - 1 != pkt[0] as usize {
        panic!("Invalid packet length");
    }
    // check type
    if pkt[2] != 0x3C {
        panic!("Invalid packet type");
    }

    // parse
    let x = &pkt[3..11].to_vec();
    let y = &pkt[11..19].to_vec();
    let z = &pkt[19..27].to_vec();
    let yaw = &pkt[27..31].to_vec();
    let pitch = &pkt[31..35].to_vec();
    let flags = &pkt[35..36].to_vec();
    let tp_id = &pkt[36..].to_vec();

    let x = f64::from_be_bytes(x.as_slice().try_into().unwrap());
    let y = f64::from_be_bytes(y.as_slice().try_into().unwrap());
    let z = f64::from_be_bytes(z.as_slice().try_into().unwrap());
    let yaw = f32::from_le_bytes(yaw.as_slice().try_into().unwrap());
    let pitch = f32::from_le_bytes(pitch.as_slice().try_into().unwrap());
    let is_abs = flags[0] == 0x00;
    let tp_id = transfer_var::var_int2uint(tp_id.to_vec())[0];

    return (x, y, z, yaw, pitch, is_abs, tp_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        //24003c400c0000000000004055866666680000c00c0000000000000000000000000000000e
        let pkt: Vec<u8> = vec![
            0x24, 0x00, 0x3c, 0x40, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x55, 0x86,
            0x66, 0x66, 0x68, 0x00, 0x00, 0xc0, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e,
        ];
        let res = parse(pkt);
        assert_eq!(
            res,
            (
                3.5,
                86.100000001490116119384765625f64,
                -3.5,
                0f32,
                0f32,
                true,
                14
            )
        );
    }
}
