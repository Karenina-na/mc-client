use crate::util::transfer_var;

pub fn parse(pkt: Vec<u8>) -> (f64, f64, f64, f32, f32, bool, i32) {
    // parse
    let x = &pkt[0..8].to_vec();
    let y = &pkt[8..16].to_vec();
    let z = &pkt[16..24].to_vec();
    let yaw = &pkt[24..28].to_vec();
    let pitch = &pkt[28..32].to_vec();
    let flags = &pkt[32..33].to_vec();
    let tp_id = &pkt[33..].to_vec();

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
            0x40, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x55, 0x86, 0x66, 0x66, 0x68,
            0x00, 0x00, 0xc0, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x0e,
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
