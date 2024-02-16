use crate::client::msg::mapper;

pub fn new(id: u8, check: bool) -> Vec<u8> {
    let mut login_plugin_response_pkt: Vec<u8> = Vec::new();
    login_plugin_response_pkt.push(0x00);
    login_plugin_response_pkt.push(mapper::LOGIN_PLUGIN_RESPONSE);
    login_plugin_response_pkt.push(id);
    login_plugin_response_pkt.push(match check {
        true => 0x01,
        false => 0x00,
    });
    login_plugin_response_pkt = [
        vec![login_plugin_response_pkt.len() as u8],
        login_plugin_response_pkt,
    ]
    .concat();
    login_plugin_response_pkt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let id: u8 = 0x01;
        let check: bool = false;
        let result = new(id, check);
        //0400020000
        let expected: Vec<u8> = vec![0x04, 0x00, 0x02, 0x01, 0x00];
        assert_eq!(result, expected);
    }
}
