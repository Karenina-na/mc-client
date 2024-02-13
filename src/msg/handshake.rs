use crate::util::transfer;

pub fn mc_handshake(protocol_version: u32, ip: &str, port: u16, login: bool) -> Vec<u8> {
    let mut handshake_pkt: Vec<u8> = Vec::new();
    handshake_pkt.push(0x00);
    handshake_pkt.append(&mut transfer::uint2var_int(vec![protocol_version]));
    handshake_pkt.append(&mut transfer::uint2var_int(vec![ip.len() as u32]));
    handshake_pkt.append(&mut ip.as_bytes().to_vec());
    handshake_pkt.append(&mut port.to_be_bytes().to_vec());
    handshake_pkt.push(match login {
        true => 0x02,
        false => 0x01,
    });
    [vec![handshake_pkt.len() as u8], handshake_pkt].concat()
}