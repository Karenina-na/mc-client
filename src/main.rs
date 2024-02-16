use env_logger::{Builder, Target};
use lazy_static::lazy_static;

mod itti;
mod msg;
mod parser;
mod util;

lazy_static! {
    static ref INIT: () = {
        std::env::set_var("RUST_LOG", "debug");
        let mut builder = Builder::from_default_env();
        builder.target(Target::Stdout);
        builder.is_test(true).init();
    };
}

async fn test() {
    let ip: &str = "127.0.0.1";
    let port: u16 = 25565;

    let mut itti = itti::basis::ITTI::new(ip.to_string(), port.to_string(), 1024, 1024);
    itti.build().await.unwrap();

    let protocol_version: i32 = 763;
    let login: bool = true;
    let handshake_pkt = msg::login::handshake::new(protocol_version, ip, port, login);
    itti.send(handshake_pkt).await.unwrap();

    let username = "Karenina";
    let login_pkt = msg::login::login_start::new(username);
    itti.send(login_pkt).await.unwrap();

    let data = itti.recv().await.unwrap();
    let threshold = parser::login::set_compression::parse(data);
    println!("server threshold: {}", threshold);

    let mut result = Vec::new();
    loop {
        let pkt = itti
            .try_recv(tokio::time::Duration::from_millis(20))
            .await
            .unwrap();
        if pkt.is_empty() {
            break;
        } else {
            result.push(pkt);
        }
    }

    println!("{:?}", result);

    let mod1 = msg::login::login_plugin_response::new(0x00, false);
    itti.send(mod1).await.unwrap();

    let mod2 = msg::login::login_plugin_response::new(0x01, false);
    itti.send(mod2).await.unwrap();

    // recv uuid
    let data = itti.recv().await.unwrap();
    let (uuid, username) = parser::login::login_success::parse(data);
    println!(
        "uuid: {}, username: {}",
        uuid.iter()
            .map(|x| format!("{:02x}", x))
            .collect::<String>(),
        username
    );

    itti.stop().await;
}

#[tokio::main]
async fn main() {
    let _ = *INIT;
    let res = util::transfer_var::var_int2uint(vec![0x1b]);
    println!("{:?}", res);
    test().await;
}
