use log::info;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

pub(crate) struct ITTI {
    reader_rx: Option<mpsc::Receiver<Vec<u8>>>,
    writer_tx: Option<mpsc::Sender<Vec<u8>>>,

    reader_num: i32, writer_num: i32,

    ip: String, port: String,
}

impl ITTI {
    /// 初始化 ITTI
    ///
    /// # Arguments
    ///
    /// * `reader_num`: 通信读进程缓冲区大小
    /// * `writer_num`: 通信写进程缓冲区大小
    ///
    /// returns: ITTI
    ///
    /// # Examples
    ///
    /// ```
    /// let itti = ITTI::new(1, 1);
    /// ```
    pub fn new(
        ip: String, port: String,
        reader_num: i32, writer_num: i32
    ) -> ITTI {
        ITTI {
            ip, port,
            reader_num, writer_num,
            reader_rx: None, writer_tx: None,
        }
    }

    pub async fn build(&mut self) -> io::Result<()>{
        let (reader_tx, reader_rx) = mpsc::channel(self.reader_num as usize);
        let (writer_tx, mut writer_rx) = mpsc::channel(self.writer_num as usize);

        self.reader_rx = Some(reader_rx);
        self.writer_tx = Some(writer_tx);

        let tcp = TcpStream::connect(format!("{}:{}", self.ip, self.port)).await?;
        let (mut reader, mut writer) = io::split(tcp);

        // reader
        tokio::spawn(async move {
            loop {
                let mut buf = vec![0; 2048];
                match reader.read(&mut buf).await {
                    Ok(n) => {
                        if n == 0 {
                            info!("reader: connection closed");
                            break;
                        }
                        let data = buf[..n].to_vec();
                        if let Err(_) = reader_tx.send(data).await {
                            info!("reader: channel closed");
                            break;
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        // writer
        tokio::spawn(async move {
            loop {
                match writer_rx.recv().await {
                    Some(data) => {
                        if let Err(_) = writer.write_all(&data).await {
                            info!("writer: connection closed");
                            break;
                        }
                    }
                    None => {
                        info!("writer: channel closed");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn send(&self, data: Vec<u8>) -> io::Result<()> {
        if let Some(writer_tx) = &self.writer_tx {
            match writer_tx.send(data).await {
                Ok(_) => {}
                Err(_) => {
                    return Err(io::Error::new(io::ErrorKind::Other, "channel closed"));
                }
            }
        }
        Ok(())
    }

    pub async fn recv(&mut self) -> io::Result<Vec<u8>> {
        if let Some(reader_rx) = &mut self.reader_rx {
            match reader_rx.recv().await {
                Some(data) => {
                    Ok(data)
                }
                None => {
                    Err(io::Error::new(io::ErrorKind::Other, "channel closed"))
                }
            }
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "channel closed"))
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::{io, spawn};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use crate::itti::basis::ITTI;

    const MSG_C2S: &str = "hello server";
    const MSG_S2C: &str = "hello client";

    async fn simple_tcp_server(){
        println!("simple_tcp_server start");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
        let (socket, _) = listener.accept().await.unwrap();
        let (mut reader, mut writer) = io::split(socket);
        println!("simple_tcp_server enter reader");

        // reader
        for _ in 0..10 {
            let mut buf = vec![0; 1024];
            match reader.read(&mut buf).await {
                Ok(n) => {
                    let data = buf[..n].to_vec();
                    println!("server recv: {:?}", String::from_utf8(data.clone()).unwrap());
                    assert_eq!(String::from_utf8(data).unwrap(), MSG_C2S);
                    if n == 0 {
                        break;
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }

        println!("simple_tcp_server enter writer");

        // writer
        for _ in 0..10 {
            let data = MSG_S2C.as_bytes().to_vec();
            writer.write_all(&data).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }

        println!("simple_tcp_server enter reader/writer");

        // reader-writer
        let mut buf = vec![0; 1024];
        for _ in 0..10 {
            let n = reader.read(&mut buf).await.unwrap();
            let data = buf[..n].to_vec();
            println!("server recv: {:?}", String::from_utf8(data.clone()).unwrap());
            assert_eq!(String::from_utf8(data).unwrap(), MSG_C2S);
            writer.write_all(MSG_S2C.as_bytes()).await.unwrap();
        }
    }

    #[tokio::test]
    async fn itti_test() {
        let server = simple_tcp_server();
        spawn(server);

        println!("itti_test start");
        let mut itti = ITTI::new("127.0.0.1".to_string(), "8080".to_string(), 1, 1);
        itti.build().await.unwrap();
        println!("itti_test connected");

        // reader
        for _ in 0..10 {
            itti.send(MSG_C2S.as_bytes().to_vec()).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }

        // writer
        for _ in 0..10 {
            let data = itti.recv().await.unwrap();
            println!("client recv: {:?}", String::from_utf8(data.clone()).unwrap());
            assert_eq!(String::from_utf8(data).unwrap(), MSG_S2C);
        }

        // reader-writer
        for _ in 0..10 {
            itti.send(MSG_C2S.as_bytes().to_vec()).await.unwrap();
            let data = itti.recv().await.unwrap();
            println!("client recv: {:?}", String::from_utf8(data.clone()).unwrap());
            assert_eq!(String::from_utf8(data).unwrap(), MSG_S2C);
        }
    }
}