use crate::util::transfer_var;

pub fn parse(pkt: Vec<u8>) -> i32 {
    let threshold = transfer_var::var_int2uint(Vec::from(&pkt[0..]));
    threshold[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mc_set_compression() {
        let pkt: Vec<u8> = vec![0x80, 0x02];
        assert_eq!(parse(pkt), 256);
    }
}
