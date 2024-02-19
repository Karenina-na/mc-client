use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Buffer {
    pub tcp_buffer: TcpBuffer,
    pub console_client_buffer: ConsoleClientBuffer,
}
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct TcpBuffer {
    pub reader: i64,
    pub writer: i64,
}
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct ConsoleClientBuffer {
    pub command: i64,
    pub response: i64,
}
