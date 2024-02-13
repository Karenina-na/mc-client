use crate::util::transfer;

pub fn mc_login_start(username: &str) -> Vec<u8> {
    let mut login_start_pkt: Vec<u8> = Vec::new();
    login_start_pkt.push(0x00);
    login_start_pkt.append(&mut transfer::uint2var_int(vec![username.len() as u32]));
    login_start_pkt.append(&mut username.as_bytes().to_vec());
    login_start_pkt.append(&mut vec![0x00]);    // uuid
    login_start_pkt = [vec![login_start_pkt.len() as u8], login_start_pkt].concat();
    login_start_pkt
}