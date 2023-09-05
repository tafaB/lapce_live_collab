use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

#[tokio::main]
pub async fn main() {
    println!("Hello from client_1");

    let url = url::Url::parse("ws://localhost:8080").unwrap();
    let (_ws_stream_1, _response_1) = connect_async(url.clone()).await.expect("Failed to connect");
    let (ws_stream, _response) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    // Create the JSON payload
    let payload = json!({
        "action": "new_change",
        "value": "B",
        "position_id": [
        {
            "number" : "2",
            "user" : "1"
        },
        {
            "number" : "3",
            "user" : "1"
        }
        ]
    });

    // Serialize the JSON payload and send it
    let message = Message::Text(serde_json::to_string(&payload).unwrap());
    write.send(message).await.expect("Failed to send data");

    loop {
        let message = read.next().await.expect("error");
        let message = message.unwrap();
        match message {
            Message::Text(text) => {
                println!("text = {}", text);
            }
            _ => {
                println!("none");
            }
        }
    }
}

