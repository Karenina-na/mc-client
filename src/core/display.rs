use log::info;
use tokio::sync::mpsc;

pub fn build_display(mut msg_rx: mpsc::Receiver<Vec<String>>) {
    // display
    tokio::spawn(async move {
        loop {
            match msg_rx.recv().await {
                Some(msg) => {
                    for line in msg {
                        println!("{}", line);
                    }
                }
                None => {
                    info!("display: channel closed");
                    break;
                }
            }
        }
    });
}
