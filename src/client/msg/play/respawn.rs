use crate::client::msg::mapper;

pub fn new() -> Vec<u8> {
    let mut respawn: Vec<u8> = Vec::new();
    respawn.push(0x00);
    respawn.push(mapper::RESPAWN);
    respawn.push(0x00);
    respawn = [vec![respawn.len() as u8], respawn].concat();
    respawn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mc_respawn() {
        let respawn_pkt = new();
        assert_eq!(respawn_pkt, vec![0x03, 0x00, 0x07, 0x00]);
    }
}
