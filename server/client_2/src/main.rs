use tokio::io::Result;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

#[tokio::main]
pub async fn main() -> Result<()> {
    println!("Hello from client_2");

    let url = url::Url::parse("ws://localhost:8080").unwrap();
    let berin  = String::from

    let (ws_stream, _response) = connect_async(url).await.expect("Failed to connect");

    let (mut write, read) = ws_stream.split();
    write.send(Message::Text("kebiana qorri".to_string())).await.expect("Failed to send data");

    let read_future = read.for_each(|message| async {
        match message {
            Ok(Message::Text(text)) => {
                println!("received: {}", text);
            }
            _ => {
            }
        }
    });

    read_future.await;

    Ok(())
}
