use crate::util;

// split tcp packet
#[allow(dead_code)]
pub fn split_tcp_packet(packet: Vec<u8>) -> (Vec<Vec<u8>>, i32, Vec<u8>) {
    let mut result = Vec::new();
    let mut index = 0;
    let packet_len_all = packet.len();
    while index < packet_len_all {
        // if not enough len for var_int
        if packet_len_all - index <= 3 {
            return (result, 0, packet[index..].to_vec());
        }
        let var_int_num = get_var_int_num(packet[index..].to_vec(), 1);
        let packet_len =
            util::transfer_var::var_int2uint(packet[index..index + var_int_num[0]].to_vec())[0]
                as usize
                + var_int_num[0];
        if index + packet_len >= packet_len_all {
            // not enough data
            let packet_data = packet[index..].to_vec();
            let len = packet_data.len() as i32;
            result.push(packet_data);
            return (result, packet_len as i32 - len, vec![]);
        }
        let packet_data = packet[index..index + packet_len].to_vec();
        result.push(packet_data);
        index += packet_len;
    }
    (result, 0, vec![])
}

// split packet
#[allow(dead_code)]
pub fn split_packet(packet: Vec<u8>, threshold: i32) -> (i32, i32, i32, Vec<u8>) {
    // check compress
    return if threshold >= 0 {
        // compress
        let var_int_num = get_var_int_num(packet.clone(), 2);
        let packet_len = util::transfer_var::var_int2uint(packet[0..var_int_num[0]].to_vec())[0];
        let data_len = util::transfer_var::var_int2uint(
            packet[var_int_num[0]..var_int_num[0] + var_int_num[1]].to_vec(),
        )[0];
        let packet_id_data = packet[var_int_num[0] + var_int_num[1]..].to_vec();
        (packet_len, data_len, -1, packet_id_data)
    } else {
        // no compress
        let var_int_num = get_var_int_num(packet.clone(), 2);
        let packet_len = util::transfer_var::var_int2uint(packet[0..var_int_num[0]].to_vec())[0];
        let packet_id = util::transfer_var::var_int2uint(
            packet[var_int_num[0]..var_int_num[0] + var_int_num[1]].to_vec(),
        )[0];
        let packet_data = packet[var_int_num[0] + var_int_num[1]..].to_vec();
        (packet_len, -1, packet_id, packet_data)
    };
}

// never used
#[allow(dead_code)]
pub fn get_var_int_num(packet: Vec<u8>, num: i32) -> Vec<usize> {
    let mut result = Vec::new();
    let mut index = 0;
    let mut last_index = 0;

    for _ in 0..num {
        loop {
            let byte = packet.get(index).cloned().unwrap_or(0);
            index += 1;

            if byte & 0x80 == 0 {
                break;
            }
        }

        result.push(index - last_index);
        last_index = index;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_tcp_packet() {
        // no compress
        let packet = vec![
            // one
            0x0a, 0x52, 0xd2, 0x06, 0x09, 0x03, 0x00, 0x00, 0x00, 0x00, 0xff, // two
            0x11, 0x5e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0xec, 0xc1, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x0a, 0x26, 0x12, // three (compress),
            0x24, 0x1b, 0x78, 0x9c, 0x63, 0x62, 0xae, 0x0f, 0x9b, 0x7a, 0xc6, 0xc0, 0x32, 0x65,
            0xce, 0xfa, 0x1e, 0x06, 0x3e, 0x81, 0x1a, 0x11, 0x0e, 0xef, 0xc4, 0xa2, 0xd4, 0xbc,
            0xcc, 0xbc, 0x44, 0x06, 0x00, 0x7c, 0xb5, 0x08, 0xbf, // four (compress)
            0x1d, 0x14, 0x78, 0x9c, 0xcb, 0x66, 0x14, 0xcc, 0xcd, 0xcc, 0x4b, 0x4d, 0x2e, 0x4a,
            0x4c, 0x2b, 0xb1, 0x2a, 0x4b, 0xcc, 0xcb, 0xcc, 0xc9, 0x49, 0x04, 0x00, 0x47, 0xab,
            0x07, 0x58, // five not enough
            0x1d, 0x14, 0x78, 0x9c, 0xcb, 0x66, 0x14, 0xcc, 0xcd, 0xcc, 0x4b, 0x4d, 0x2e, 0x4a,
            0x4c, 0x2b, 0xb1, 0x2a, 0x4b, 0xcc, 0xcb, 0xcc, 0xc9, 0x49, 0x04, 0x00, 0x47,
        ];
        let (res, len, _) = split_tcp_packet(packet);
        assert_eq!(res.len(), 5);
        assert_eq!(
            res[0],
            vec![0x0a, 0x52, 0xd2, 0x06, 0x09, 0x03, 0x00, 0x00, 0x00, 0x00, 0xff,]
        );
        assert_eq!(
            res[1],
            vec![
                0x11, 0x5e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0xec, 0xc1, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x0a, 0x26, 0x12
            ]
        );
        assert_eq!(
            res[2],
            vec![
                0x24, 0x1b, 0x78, 0x9c, 0x63, 0x62, 0xae, 0x0f, 0x9b, 0x7a, 0xc6, 0xc0, 0x32, 0x65,
                0xce, 0xfa, 0x1e, 0x06, 0x3e, 0x81, 0x1a, 0x11, 0x0e, 0xef, 0xc4, 0xa2, 0xd4, 0xbc,
                0xcc, 0xbc, 0x44, 0x06, 0x00, 0x7c, 0xb5, 0x08, 0xbf
            ]
        );
        assert_eq!(
            res[3],
            vec![
                0x1d, 0x14, 0x78, 0x9c, 0xcb, 0x66, 0x14, 0xcc, 0xcd, 0xcc, 0x4b, 0x4d, 0x2e, 0x4a,
                0x4c, 0x2b, 0xb1, 0x2a, 0x4b, 0xcc, 0xcb, 0xcc, 0xc9, 0x49, 0x04, 0x00, 0x47, 0xab,
                0x07, 0x58
            ]
        );
        assert_eq!(
            res[4],
            vec![
                0x1d, 0x14, 0x78, 0x9c, 0xcb, 0x66, 0x14, 0xcc, 0xcd, 0xcc, 0x4b, 0x4d, 0x2e, 0x4a,
                0x4c, 0x2b, 0xb1, 0x2a, 0x4b, 0xcc, 0xcb, 0xcc, 0xc9, 0x49, 0x04, 0x00, 0x47
            ]
        );
        assert_eq!(len, 3);
        let packet = vec![
            0x1d, 0x14, 0x78, 0x9c, 0xcb, 0x66, 0x14, 0xcc, 0xcd, 0xcc, 0x4b, 0x4d, 0x2e, 0x4a,
            0x4c, 0x2b, 0xb1, 0x2a, 0x4b, 0xcc, 0xcb, 0xcc, 0xc9, 0x49, 0x04, 0x00, 0x47, 0xab,
            0x07, 0x58,
        ];
        let (_, len, _) = split_tcp_packet(packet);
        assert_eq!(len, 0);
        let packet = vec![
            // one
            0x0a, 0x52, 0xd2, 0x06, 0x09, 0x03, 0x00, 0x00, 0x00, 0x00, 0xff, // two
            0x11, 0x5e,
        ];
        let (_, len, res) = split_tcp_packet(packet);
        assert_eq!(len, 0);
        assert_eq!(res, vec![0x11, 0x5e]);
    }

    #[test]
    fn test_split_packet() {
        let packet = vec![
            0x90, 0xb8, 0x02, 0x24, 0xff, 0xff, 0xff, 0xf6, 0xff, 0xff, 0xff, 0xf7, 0x0a, 0x00,
            0x00, 0x0c, 0x00, 0x0f, 0x4d, 0x4f, 0x54, 0x49, 0x4f, 0x4e, 0x5f, 0x42, 0x4c, 0x4f,
            0x43, 0x4b, 0x49, 0x4e, 0x47, 0x00, 0x00, 0x00, 0x25, 0x21, 0x90, 0xa8, 0x54, 0x2a,
            0x15, 0x0a, 0x85, 0x22, 0xd1, 0x48, 0xa4, 0x52, 0x25, 0x10, 0x87, 0x21, 0x50, 0xa8,
        ];
        let (packet_len, data_len, packet_id, packet_data) = split_packet(packet, -1);
        assert_eq!(packet_len, 39952);
        assert_eq!(data_len, -1);
        assert_eq!(packet_id, 36);
        assert_eq!(
            packet_data,
            vec![
                0xff, 0xff, 0xff, 0xf6, 0xff, 0xff, 0xff, 0xf7, 0x0a, 0x00, 0x00, 0x0c, 0x00, 0x0f,
                0x4d, 0x4f, 0x54, 0x49, 0x4f, 0x4e, 0x5f, 0x42, 0x4c, 0x4f, 0x43, 0x4b, 0x49, 0x4e,
                0x47, 0x00, 0x00, 0x00, 0x25, 0x21, 0x90, 0xa8, 0x54, 0x2a, 0x15, 0x0a, 0x85, 0x22,
                0xd1, 0x48, 0xa4, 0x52, 0x25, 0x10, 0x87, 0x21, 0x50, 0xa8,
            ]
        );

        let packet = vec![
            0x0a, 0x52, 0xd2, 0x06, 0x09, 0x03, 0x00, 0x00, 0x00, 0x00, 0xff,
        ];
        let (packet_len, data_len, packet_id, packet_data) = split_packet(packet, -1);
        assert_eq!(packet_len, 10);
        assert_eq!(data_len, -1);
        assert_eq!(packet_id, 82);
        assert_eq!(
            packet_data,
            vec![0xd2, 0x06, 0x09, 0x03, 0x00, 0x00, 0x00, 0x00, 0xff]
        );
    }

    #[test]
    fn test_get_var_int_num() {
        let packet = vec![
            0x0b, 0x00, 0x08, 0x4b, 0x61, 0x72, 0x65, 0x6e, 0x69, 0x6e, 0x61, 0x00,
        ];
        let res = get_var_int_num(packet, 2);
        assert_eq!(res, vec![1, 1]);
        let packet = vec![
            0x95, 0xb3, 0x02, 0x28, 0x00, 0x00, 0x03, 0x52, 0x00, 0x00, 0xff, 0x03, 0x13, 0x6d,
            0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74, 0x3a, 0x6f, 0x76, 0x65, 0x72, 0x77,
            0x6f, 0x72, 0x6c, 0x64, 0x14, 0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74,
            0x3a, 0x74, 0x68, 0x65, 0x5f, 0x6e, 0x65, 0x74, 0x68, 0x65, 0x72, 0x11, 0x6d, 0x69,
            0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74, 0x3a, 0x74, 0x68, 0x65, 0x5f, 0x65, 0x6e,
            0x64, 0x0a, 0x00, 0x00, 0x0a, 0x00, 0x16, 0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61,
            0x66, 0x74, 0x3a, 0x74, 0x72, 0x69, 0x6d, 0x5f, 0x70, 0x61, 0x74, 0x74, 0x65, 0x72,
            0x6e, 0x08, 0x00, 0x04, 0x74, 0x79, 0x70, 0x65, 0x00, 0x16, 0x6d, 0x69, 0x6e, 0x65,
        ];
        let res = get_var_int_num(packet, 2);
        assert_eq!(res, vec![3, 1]);
        let packet = vec![
            0xf9, 0xa6, 0x08, 0x6d, 0x96, 0x09, 0x19, 0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61,
            0x66, 0x74, 0x3a, 0x63, 0x72, 0x61, 0x66, 0x74, 0x69, 0x6e, 0x67, 0x5f, 0x73, 0x68,
            0x61, 0x70, 0x65, 0x64, 0x16, 0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74,
            0x3a, 0x77, 0x68, 0x69, 0x74, 0x65, 0x5f, 0x62, 0x61, 0x6e, 0x6e, 0x65, 0x72, 0x03,
            0x03, 0x06, 0x62, 0x61, 0x6e, 0x6e, 0x65, 0x72, 0x03, 0x01, 0x01, 0xb4, 0x01, 0x01,
            0x00, 0x01, 0x01, 0xb4, 0x01, 0x01, 0x00, 0x01, 0x01, 0xb4, 0x01, 0x01, 0x00, 0x01,
            0x01, 0xb4, 0x01, 0x01, 0x00, 0x01, 0x01, 0xb4, 0x01, 0x01, 0x00, 0x01, 0x01, 0xb4,
            0x01, 0x01, 0x00, 0x00, 0x01, 0x01, 0xa7, 0x06, 0x01, 0x00, 0x00, 0x01, 0xbf, 0x08,
        ];
        let res = get_var_int_num(packet, 2);
        assert_eq!(res, vec![3, 1]);
        let packet = vec![0xf9, 0xa6, 0x08, 0x95, 0xb3, 0x02, 0x00, 0x01];
        let res = get_var_int_num(packet, 2);
        assert_eq!(res, vec![3, 3]);
    }
}
