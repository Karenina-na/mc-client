use crate::client::msg::mapper;
use crate::util::transfer_var;

pub fn new(id: i32, compress: bool) -> Vec<u8> {
    let mut confirmed_tp_pkt: Vec<u8> = Vec::new();
    if compress {
        confirmed_tp_pkt.push(0x00);
    }
    confirmed_tp_pkt.push(mapper::CONFIRM_TP);
    let id = transfer_var::uint2var_int(vec![id]);
    confirmed_tp_pkt = [confirmed_tp_pkt, id].concat();
    confirmed_tp_pkt = [vec![confirmed_tp_pkt.len() as u8], confirmed_tp_pkt].concat();
    confirmed_tp_pkt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mc_confirmed_tp_compress() {
        let id = 0x0e;
        let confirmed_tp_pkt = new(id, true);
        // 0300000e
        assert_eq!(confirmed_tp_pkt, vec![0x03, 0x00, 0x00, 0x0e]);
    }

    #[test]
    fn test_mc_confirmed_tp_no_compress() {
        let id = 0x0e;
        let confirmed_tp_pkt = new(id, false);
        // 0100000e
        assert_eq!(confirmed_tp_pkt, vec![0x02, 0x00, 0x0e]);
    }
}
