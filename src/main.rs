mod msg;
mod util;

use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let protocol_version = 763;
    let ip = "localhost";
    let port: u16 = 25565;
    let login = true;
    let username = "Karenina";
    let mut stream = TcpStream::connect(format!("{}:{}", ip, port)).unwrap();

    // handshake
    let handshake_pkt = msg::handshake::mc_handshake(protocol_version, ip, port, login);
    stream.write(&handshake_pkt).unwrap();

    // login start
    let login_start_pkt = msg::login_start::mc_login_start(username);
    stream.write(&login_start_pkt).unwrap();

    // read max length
    let mut buf = [0; 1024];
    let n = stream.read(&mut buf).unwrap();
    let res = util::transfer::var_int2uint(buf[0..n].to_vec());
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
        let point = point_pkt + 1;
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
        let data = msg::login_mod_check::mc_login_mod_check(id, false);
        stream.write(&data).unwrap();
        point_pkt += len_pkt + 1;
    }

    // uuid
    let res;
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
