pub fn uint2var_int(n: Vec<u32>) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();
    for i in n {
        let mut b: Vec<u8> = Vec::new();
        let mut j = i;
        while j >= 0x80 {
            b.push((j & 0x7F) as u8 | 0x80);
            j >>= 7;
        }
        b.push(j as u8);
        res.append(&mut b);
    }
    res
}

pub fn var_int2uint(b: Vec<u8>) -> Vec<u32> {
    let mut res: Vec<u32> = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let mut j = 0;
        let mut k = 0;
        while (b[i] & 0x80) != 0 {
            j |= ((b[i] & 0x7F) as u32) << k;
            k += 7;
            i += 1;
        }
        j |= (b[i] as u32) << k;
        res.push(j);
        i += 1;
    }
    res
}
