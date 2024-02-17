use crate::client::msg::mapper;

pub fn new(compress: bool) -> Vec<u8> {
    let mut respawn: Vec<u8> = Vec::new();
    if compress {
        respawn.push(0x00);
    }
    respawn.push(mapper::RESPAWN);
    respawn.push(0x00);
    respawn = [vec![respawn.len() as u8], respawn].concat();
    respawn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mc_respawn_compress() {
        let respawn_pkt = new(true);
        assert_eq!(respawn_pkt, vec![0x03, 0x00, 0x07, 0x00]);
    }

    #[test]
    fn test_mc_respawn_no_compress() {
        let respawn_pkt = new(false);
        assert_eq!(respawn_pkt, vec![0x02, 0x07, 0x00]);
    }
}
