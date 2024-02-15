use crate::util::transfer;

pub fn new(protocol_version: i32, ip: &str, port: u16, login: bool) -> Vec<u8> {
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
        let protocol_version: i32 = 763;
        let ip: &str = "127.0.0.1";
        let port: u16 = 25565;
        let login: bool = true;

        // 1000fb05093132372e302e302e3163dd02
        let expected: Vec<u8> = vec![
            0x10, 0x00, 0xFB, 0x05, 0x09, 0x31, 0x32, 0x37, 0x2E, 0x30, 0x2E, 0x30, 0x2E, 0x31,
            0x63, 0xDD, 0x02,
        ];
        assert_eq!(new(protocol_version, ip, port, login), expected);
    }
}
