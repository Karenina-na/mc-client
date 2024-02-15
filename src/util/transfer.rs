#[allow(dead_code)]
pub fn uint2var_int(n: Vec<i32>) -> Vec<u8> {
    let n: Vec<u32> = n.iter().map(|x| *x as u32).collect();
    let mut res: Vec<u8> = Vec::new();
    for mut value in n {
        while value > 0x7F {
            res.push((value & 0x7F) as u8 | 0x80);
            value >>= 7;
        }
        res.push(value as u8);
    }
    res
}

#[allow(dead_code)]
pub fn var_int2uint(b: Vec<u8>) -> Vec<i32> {
    let mut res: Vec<u32> = Vec::new();
    let mut value: u32 = 0;
    let mut position: u32 = 0;
    for i in b {
        value |= ((i & 0x7F) as u32) << position;
        if (i & 0x80) == 0 {
            res.push(value);
            value = 0;
            position = 0;
        } else {
            position += 7;
            if position >= 32 {
                panic!("VarInt is too big");
            }
        }
    }
    res.iter().map(|x| *x as i32).collect()
}

#[allow(dead_code)]
pub fn uint2var_long(n: Vec<i64>) -> Vec<u8> {
    let n: Vec<u64> = n.iter().map(|x| *x as u64).collect();
    let mut res: Vec<u8> = Vec::new();
    for mut value in n {
        while value > 0x7F {
            res.push((value & 0x7F) as u8 | 0x80);
            value >>= 7;
        }
        res.push(value as u8);
    }
    res
}

#[allow(dead_code)]
pub fn var_long2uint(b: Vec<u8>) -> Vec<i64> {
    let mut res: Vec<u64> = Vec::new();
    let mut value: u64 = 0;
    let mut position: u64 = 0;
    for i in b {
        value |= ((i & 0x7F) as u64) << position;
        if (i & 0x80) == 0 {
            res.push(value);
            value = 0;
            position = 0;
        } else {
            position += 7;
            if position >= 64 {
                panic!("VarLong is too big");
            }
        }
    }
    res.iter().map(|x| *x as i64).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uint2var_int() {
        let n: Vec<i32> = vec![0, 1, 2, 127, 128, 255, 25565, 2097151, -1, -2147483648];
        let res: Vec<u8> = uint2var_int(n);
        assert_eq!(
            res,
            vec![
                0x00, 0x01, 0x02, 0x7F, 0x80, 0x01, 0xFF, 0x01, 0xDD, 0xC7, 0x01, 0xFF, 0xFF, 0x7F,
                0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0x80, 0x80, 0x80, 0x80, 0x08
            ]
        );
    }

    #[test]
    fn test_var_int2uint() {
        let b: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x7F, 0x80, 0x01, 0xFF, 0x01, 0xDD, 0xC7, 0x01, 0xFF, 0xFF, 0x7F,
            0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0x80, 0x80, 0x80, 0x80, 0x08,
        ];
        let res: Vec<i32> = var_int2uint(b);
        assert_eq!(
            res,
            vec![0, 1, 2, 127, 128, 255, 25565, 2097151, -1, -2147483648]
        );
    }

    #[test]
    fn test_uint2var_long() {
        let n: Vec<i64> = vec![
            0,
            1,
            2,
            127,
            128,
            255,
            2147483647,
            9223372036854775807,
            -1,
            -2147483648,
            -9223372036854775808,
        ];
        let res: Vec<u8> = uint2var_long(n);
        assert_eq!(
            res,
            vec![
                0x00, 0x01, 0x02, 0x7F, 0x80, 0x01, 0xFF, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x07, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0x01, 0x80, 0x80, 0x80, 0x80, 0xF8, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
                0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01
            ]
        );
    }

    #[test]
    fn test_var_long2uint() {
        let b: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x7F, 0x80, 0x01, 0xFF, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x07, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0x01, 0x80, 0x80, 0x80, 0x80, 0xF8, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01,
        ];
        let res: Vec<i64> = var_long2uint(b);
        assert_eq!(
            res,
            vec![
                0,
                1,
                2,
                127,
                128,
                255,
                2147483647,
                9223372036854775807,
                -1,
                -2147483648,
                -9223372036854775808
            ]
        );
    }
}
