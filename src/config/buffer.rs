use lazy_static::lazy_static;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct Buffer {
    #[validate]
    pub tcp_buffer: TcpBuffer,
    #[validate]
    pub console_client_buffer: ConsoleClientBuffer,
}
#[derive(Deserialize, Validate)]
pub struct TcpBuffer {
    pub reader: i64,
    pub writer: i64,
}
#[derive(Deserialize, Validate)]
pub struct ConsoleClientBuffer {
    pub command: i64,
    pub response: i64,
}

lazy_static!{
    static ref IS_DIGIT: regex::Regex = regex::Regex::new(r"^\d+$").unwrap();
}