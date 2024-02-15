use crate::util::transfer;

pub fn new(username: &str) -> Vec<u8> {
    let mut login_start_pkt: Vec<u8> = Vec::new();
    login_start_pkt.push(0x00);
    login_start_pkt.append(&mut transfer::uint2var_int(vec![username.len() as i32]));
    login_start_pkt.append(&mut username.as_bytes().to_vec());
    login_start_pkt.append(&mut vec![0x00]); // uuid
    login_start_pkt = [vec![login_start_pkt.len() as u8], login_start_pkt].concat();
    login_start_pkt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mc_login_start() {
        let username = "test";
        let login_start_pkt = new(username);
        // 0700047465737400
        assert_eq!(
            login_start_pkt,
            vec![0x07, 0x00, 0x04, 0x74, 0x65, 0x73, 0x74, 0x00]
        );
    }
}
