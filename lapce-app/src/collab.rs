use futures::{stream::SplitStream, SinkExt, StreamExt};
use rust_decimal::Decimal;
use std::{
    fs::{File, OpenOptions},
    io::{self, Error, Write},
    path::Path,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::crdt::{comp_character, comp_id, Character, Id};
use serde::{Deserialize, Serialize};

use serde_json::json;
use tokio::io::Result;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{protocol::Message, WebSocket},
    WebSocketStream,
};

//   {
//     "action": "new_change",
//     "value": "B",
//     "position_id": [
//       {
//         "number": "1",
//         "user": "1"
//       },
//       {
//         "number": "1",
//         "user": "2"
//       }
//     ]
//   }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChangeResponse {
    pub action: String,
    pub value: String,
    pub position_id: Vec<Position>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Position {
    pub number: String,
    pub user: String,
}

fn write_file(x: &str, y: &str) {
    let mut file = File::create(y).unwrap();
    file.write_all(x.as_bytes())
        .expect("Failed to write to file");
    file.flush().expect("Failed to flush file");
}

fn bering(data: &str, file_path: &str) {
    let mut file = OpenOptions::new().append(true).open(file_path).unwrap();
    writeln!(&mut file, "{}", data).unwrap();
}

pub fn generate_initial_crdt(
    file_content: String,
    user_id: u32,
) -> Vec<Vec<Character>> {
    let mut crdt_file_copy: Vec<Vec<Character>> = Vec::new();
    let mut line_count: usize = 0;
    let mut character_count: usize = 1;
    crdt_file_copy.push(Vec::new());
    let mut id_vec: Vec<Decimal> = Vec::new();
    for _ in file_content.chars() {
        let number_str = character_count.to_string();
        let res = Decimal::from_str(&format!("0.{}", number_str)).unwrap();
        let res_string = res.to_string();
        let remove_extra_zero =
            res_string.trim_end_matches('0').trim_end_matches('.');
        let trimmed_res = Decimal::from_str(remove_extra_zero).unwrap();
        id_vec.push(trimmed_res);
        character_count += 1;
        if character_count % 10 == 0 {
            character_count += 1;
        }
    }
    id_vec.sort();
    character_count = 0;
    for c in file_content.chars() {
        let mut new_pos_id: Vec<Id> = Vec::new();
        let decimal_str = id_vec[character_count].fract().to_string();
        let decimal_str = decimal_str.trim_start_matches("0.");
        let digits: Vec<u32> = decimal_str
            .chars()
            .map(|c| c.to_digit(10).unwrap())
            .collect();
        for i in digits {
            new_pos_id.push(Id {
                number: i,
                user: user_id,
            });
        }
        let new_char = Character {
            value: c,
            pos_id: new_pos_id,
            action_id: 0,
        };
        crdt_file_copy[line_count].push(new_char);
        if c == '\n' {
            line_count += 1;
            crdt_file_copy.push(Vec::new());
        }
        character_count += 1;
    }
    return crdt_file_copy;
}

pub async fn collab_session_mainloop(
    file_path: &str,
    crdt_file_copy_2d: Vec<Vec<Character>>,
) {
    let crdt_file_copy: Vec<Character> =
        crdt_file_copy_2d.into_iter().flatten().collect();

    let url = url::Url::parse("ws://localhost:8080").unwrap();
    write_file("we_are\n", "kebiana_bering_tafa.txt");
    let (ws_stream, _response) =
        connect_async(url).await.expect("Failed to connect");
    write_file("we_are_in\n", "kebiana_bering_tafa.txt");
    let (_write, read) = ws_stream.split();
    write_file("we_are_in\n", "kebiana_bering_tafa.txt");
    let crdt_file_copy_mutex = Arc::new(Mutex::new(crdt_file_copy));

    let read_future = read.for_each(|message| {
        let crdt_file_copy_mutex = Arc::clone(&crdt_file_copy_mutex);

        async move {
            match message {
                Ok(Message::Text(text)) => {
                    write_file("we_are_in_again\n", "kebiana_bering_tafa.txt");
                    let response: ChangeResponse = serde_json::from_str(&text)
                        .expect("Failed to parse response");
                    let mut position_identifier: Vec<Id> = Vec::new();
                    for i in response.position_id {
                        let elem = Id {
                            number: i.number.parse().unwrap(),
                            user: i.user.parse().unwrap(),
                        };
                        position_identifier.push(elem);
                    }
                    let new_character = Character {
                        value: response
                            .value
                            .chars()
                            .next()
                            .expect("error converting to char from string"),
                        pos_id: position_identifier,
                        action_id: 0,
                    };

                    let mut crdt_file_copy = crdt_file_copy_mutex.lock().unwrap();
                    crdt_file_copy.push(new_character);
                    crdt_file_copy.sort_by(comp_character);

                    let mut content = String::new();
                    for i in &*crdt_file_copy {
                        content.push(i.value);
                    }
                    write_file(&content, "kebiana_bering_tafa.txt");
                    write_file(&content, file_path)
                }
                _ => {}
            }
        }
    });

    read_future.await;
}