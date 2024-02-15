use crate::util::transfer;

pub fn mc_handshake(protocol_version: i32, ip: &str, port: u16, login: bool) -> Vec<u8> {
    let mut handshake_pkt: Vec<u8> = Vec::new();
    handshake_pkt.push(0x00);
    handshake_pkt.append(&mut transfer::uint2var_int(vec![protocol_version]));
    handshake_pkt.append(&mut transfer::uint2var_int(vec![ip.len() as i32]));
    handshake_pkt.append(&mut ip.as_bytes().to_vec());
    handshake_pkt.append(&mut port.to_be_bytes().to_vec());
    handshake_pkt.push(match login {
        true => 0x02,
        false => 0x01,
    });
    [vec![handshake_pkt.len() as u8], handshake_pkt].concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mc_handshake() {
        let protocol_version: i32 = 754;
        let ip: &str = "localhost";
        let port: u16 = 25565;
        let login: bool = true;
        let expected: Vec<u8> = vec![
            0x00, 0x04, 0x00, 0x00, 0x00, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73,
            0x74, 0x00, 0x00, 0x01, 0x01,
        ];
        assert_eq!(mc_handshake(protocol_version, ip, port, login), expected);
    }
}