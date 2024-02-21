use crate::util;

pub fn parse(pkt: Vec<u8>) -> (String, i32, String, bool, String) {
    // parse
    let msg_n_num = util::split::get_var_int_num(pkt.clone(), 1);
    let msg_n = util::transfer_var::var_int2uint(pkt[0..msg_n_num[0]].to_vec())[0] as usize;
    let msg = pkt[msg_n_num[0]..msg_n_num[0] + msg_n].to_vec();

    let chat_type_num = util::split::get_var_int_num(pkt[msg_n_num[0] + msg_n..].to_vec(), 1);
    let chat_type = util::transfer_var::var_int2uint(
        pkt[msg_n_num[0] + msg_n..msg_n_num[0] + msg_n + chat_type_num[0]].to_vec(),
    )[0];

    let chat_type_name_n_num =
        util::split::get_var_int_num(pkt[msg_n_num[0] + msg_n + chat_type_num[0]..].to_vec(), 1);
    let chat_type_name_n = util::transfer_var::var_int2uint(
        pkt[msg_n_num[0] + msg_n + chat_type_num[0]
            ..msg_n_num[0] + msg_n + chat_type_num[0] + chat_type_name_n_num[0]]
            .to_vec(),
    )[0] as usize;
    let chat_type_name = pkt[msg_n_num[0] + msg_n + chat_type_num[0] + chat_type_name_n_num[0]
        ..msg_n_num[0] + msg_n + chat_type_num[0] + chat_type_name_n_num[0] + chat_type_name_n]
        .to_vec();

    let has_target_name = pkt[msg_n_num[0]
        + msg_n
        + chat_type_num[0]
        + chat_type_name_n_num[0]
        + chat_type_name_n
        ..msg_n_num[0] + msg_n + chat_type_num[0] + chat_type_name_n_num[0] + chat_type_name_n + 1]
        [0]
        == 0x01;

    // convert
    let msg = String::from_utf8(msg).unwrap();
    let chat_type_name = String::from_utf8(chat_type_name).unwrap();

    if has_target_name {
        let target_name_n_num = util::split::get_var_int_num(
            pkt[msg_n_num[0]
                + msg_n
                + chat_type_num[0]
                + chat_type_name_n_num[0]
                + chat_type_name_n
                + 1..]
                .to_vec(),
            1,
        );
        let target_name_n = util::transfer_var::var_int2uint(
            pkt[msg_n_num[0]
                + msg_n
                + chat_type_num[0]
                + chat_type_name_n_num[0]
                + chat_type_name_n
                + 1
                ..msg_n_num[0]
                    + msg_n
                    + chat_type_num[0]
                    + chat_type_name_n_num[0]
                    + chat_type_name_n
                    + 1
                    + target_name_n_num[0]]
                .to_vec(),
        )[0] as usize;
        let target_name = pkt[msg_n_num[0]
            + msg_n
            + chat_type_num[0]
            + chat_type_name_n_num[0]
            + chat_type_name_n
            + 1
            + target_name_n_num[0]
            ..msg_n_num[0]
                + msg_n
                + chat_type_num[0]
                + chat_type_name_n_num[0]
                + chat_type_name_n
                + 1
                + target_name_n_num[0]
                + target_name_n]
            .to_vec();
        let target_name = String::from_utf8(target_name).unwrap();
        return (msg, chat_type, chat_type_name, has_target_name, target_name);
    }

    (
        msg,
        chat_type,
        chat_type_name,
        has_target_name,
        String::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let pkt = vec![
            0x13, 0x7b, 0x22, 0x74, 0x65, 0x78, 0x74, 0x22, 0x3a, 0x22, 0x66, 0x75, 0x63, 0x6b,
            0x20, 0x79, 0x6f, 0x75, 0x22, 0x7d, 0x02, 0x11, 0x7b, 0x22, 0x74, 0x65, 0x78, 0x74,
            0x22, 0x3a, 0x22, 0x53, 0x65, 0x72, 0x76, 0x65, 0x72, 0x22, 0x7d, 0x01, 0x04, 0x74,
            0x65, 0x78, 0x74,
        ];
        let (msg, chat_type, chat_type_name, has_target_name, target_name) = parse(pkt);
        assert_eq!(msg, "{\"text\":\"fuck you\"}");
        assert_eq!(chat_type, 0x02);
        assert_eq!(chat_type_name, "{\"text\":\"Server\"}");
        assert_eq!(has_target_name, true);
        assert_eq!(target_name, "text");
    }
}
