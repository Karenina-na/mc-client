use std::io::{Read, Write};
use std::net::TcpStream;

fn uint2var_int(n: Vec<u32>) -> Vec<u8> {
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

fn var_int2uint(b: Vec<u8>) -> Vec<u32> {
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

fn mc_handshake(protocol_version: u32, ip: &str, port: u16, login: bool) -> Vec<u8> {
    let mut handshake_pkt: Vec<u8> = Vec::new();
    handshake_pkt.push(0x00);
    handshake_pkt.append(&mut uint2var_int(vec![protocol_version]));
    handshake_pkt.append(&mut uint2var_int(vec![ip.len() as u32]));
    handshake_pkt.append(&mut ip.as_bytes().to_vec());
    handshake_pkt.append(&mut port.to_be_bytes().to_vec());
    handshake_pkt.push(match login {
        true => 0x02,
        false => 0x01,
    });
    [vec![handshake_pkt.len() as u8], handshake_pkt].concat()
}

fn mc_login_start(username: &str) -> Vec<u8> {
    let mut login_start_pkt: Vec<u8> = Vec::new();
    login_start_pkt.push(0x00);
    login_start_pkt.append(&mut uint2var_int(vec![username.len() as u32]));
    login_start_pkt.append(&mut username.as_bytes().to_vec());
    login_start_pkt.append(&mut vec![0x00]);    // uuid
    login_start_pkt = [vec![login_start_pkt.len() as u8], login_start_pkt].concat();
    login_start_pkt
}

fn mc_login_mod_check(id: u8, check: bool) -> Vec<u8> {
    let mut login_plugin_response_pkt: Vec<u8> = Vec::new();
    login_plugin_response_pkt.push(0x00);
    login_plugin_response_pkt.push(0x02);
    login_plugin_response_pkt.push(id);
    login_plugin_response_pkt.push(match check {
        true => 0x01,
        false => 0x00,
    });
    login_plugin_response_pkt = [vec![login_plugin_response_pkt.len() as u8], login_plugin_response_pkt].concat();
    login_plugin_response_pkt
}

fn main() {
    let protocol_version = 763;
    let ip = "localhost";
    let port: u16 = 25565;
    let login = true;
    let username = "Karenina";
    let mut stream = TcpStream::connect(format!("{}:{}", ip, port)).unwrap();

    // handshake
    let handshake_pkt = mc_handshake(protocol_version, ip, port, login);
    stream.write(&handshake_pkt).unwrap();

    // login start
    let login_start_pkt = mc_login_start(username);
    stream.write(&login_start_pkt).unwrap();

    // read max length
    let mut buf = [0; 1024];
    let n = stream.read(&mut buf).unwrap();
    let res = var_int2uint(buf[0..n].to_vec());
    let max_len = res[res.len() -1] as usize;
    println!("max-len: {:?}", max_len);

    // get mod check
    stream.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap();
    let mut mod_check_pkt = Vec::new();
    loop {
        let mut buf = [0; 1024];
        match stream.read(&mut buf) {
            Ok(n) => {
                let mut res = buf[0..n].to_vec();
                mod_check_pkt.append(&mut res);
            },
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    break;
                }else {
                    panic!("error: {:?}", e);
                }
            }
        }
    }

    // split and process
    let mut flag = true;
    let mut point_pkt = 0;
    while point_pkt < mod_check_pkt.len() {
        let len_pkt = mod_check_pkt[point_pkt] as usize;
        let mut point = point_pkt + 1;
        // 0x04
        if mod_check_pkt[point] != 0x00 && mod_check_pkt[point + 1] != 0x04{
            flag = false;
            println!("error: {:?}", mod_check_pkt.iter().map(|x| format!("{:02X}", x)).collect::<String>());
            break;
        }
        // id
        let id = mod_check_pkt[point + 2];
        // data
        let res = mod_check_pkt[point + 3..point + len_pkt].to_vec();
        let data = String::from_utf8(res).unwrap();
        println!("id: {:?}, data: {:?}", id, data);
        // send
        let data = mc_login_mod_check(id, false);
        stream.write(&data).unwrap();
        point_pkt += len_pkt + 1;
    }

    // 接受uuid
    let mut res;
    if flag {
        buf = [0; 1024];
        let n = stream.read(&mut buf).unwrap();
        res = buf[0..n].to_vec();
    }else{
        res = mod_check_pkt;
    }
    println!("{:?}", res.iter().map(|x| format!("{:02X}", x)).collect::<String>());

    stream.shutdown(std::net::Shutdown::Both).unwrap();
}
