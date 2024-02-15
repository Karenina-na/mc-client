pub fn mc_login_mod_check(id: u8, check: bool) -> Vec<u8> {
    let mut login_plugin_response_pkt: Vec<u8> = Vec::new();
    login_plugin_response_pkt.push(0x00);
    login_plugin_response_pkt.push(0x02);
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
