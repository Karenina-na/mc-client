use crate::core::msg::mapper;
use crate::util::transfer_var;

pub fn new(username: String, uuid: Vec<u8>) -> Vec<u8> {
    let mut login_start_pkt: Vec<u8> = Vec::new();
    login_start_pkt.push(mapper::LOGIN_START);
    login_start_pkt.append(&mut transfer_var::uint2var_int(vec![username.len() as i32]));
    login_start_pkt.append(&mut username.as_bytes().to_vec());
    // uuid
    if uuid.len() == 0 {
        login_start_pkt.append(&mut vec![0x00]);
    } else {
        login_start_pkt.append(&mut vec![0x01]);
        login_start_pkt.append(&mut uuid.clone());
    }
    login_start_pkt = [vec![login_start_pkt.len() as u8], login_start_pkt].concat();
    login_start_pkt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_not_uuid() {
        let username = "test".to_string();
        let login_start_pkt = new(username, vec![]);
        // 0700047465737400
        assert_eq!(
            login_start_pkt,
            vec![0x07, 0x00, 0x04, 0x74, 0x65, 0x73, 0x74, 0x00]
        );
    }

    #[test]
    fn test_new_uuid() {
        let username = "Karenina-na".to_string();
        let uuid = vec![
            0x65, 0x63, 0x2e, 0x9d, 0x20, 0xad, 0x47, 0x57, 0x95, 0x90, 0x3a, 0xd8, 0x1c, 0x2f,
            0x28, 0xe6,
        ];
        let login_start_pkt = new(username, uuid);
        assert_eq!(
            login_start_pkt,
            vec![
                0x1e, 0x00, 0x0b, 0x4b, 0x61, 0x72, 0x65, 0x6e, 0x69, 0x6e, 0x61, 0x2d, 0x6e, 0x61,
                0x01, 0x65, 0x63, 0x2e, 0x9d, 0x20, 0xad, 0x47, 0x57, 0x95, 0x90, 0x3a, 0xd8, 0x1c,
                0x2f, 0x28, 0xe6
            ]
        );
    }
}
