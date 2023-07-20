use futures::{stream::SplitStream, SinkExt, StreamExt, lock};
use rust_decimal::Decimal;
use std::{
    fs::{File, OpenOptions},
    io::{self, Error, Write},
    path::Path,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::crdt::{comp_character, comp_id, Character, Id, PrevNextCharacter, generate_pos_id,find_prev_next};
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
    pub curr_character: char,
    pub curr_col: usize,
    pub curr_row: usize,
}

fn write_file(x: &str, y: &str) {
    let mut file = File::create(y).unwrap();
    file.write_all(x.as_bytes())
        .expect("Failed to write to file");
    file.flush().expect("Failed to flush file");
}

fn bering(data: String, file_path: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .unwrap();
    file.write_all(data.as_bytes()).expect("failed to write");
    file.flush().expect("failed to flush file");
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
    user_id: u32,
    ) {
    let url = url::Url::parse("ws://localhost:8080").unwrap();
    let (ws_stream, _response) =
        connect_async(url).await.expect("Failed to connect");
    let (_write, mut read) = ws_stream.split();
    let crdt_file_copy_2d_mutex = Arc::new(Mutex::new(crdt_file_copy_2d));

    loop {
        let message = read.next().await.expect("Failed Reading from Websocket");
        let message = message.unwrap();
        match message {
            Message::Text(text) => {
                let response_json: ChangeResponse = serde_json::from_str(&text)
                    .expect("Failed to parse response");
                let temp = format!("\n just received : {} -> {} : {}", response_json.curr_character, response_json.curr_row, response_json.curr_col);
                bering(temp, "bering_tafa.txt");
                let mut locked_data = crdt_file_copy_2d_mutex.lock().unwrap();
                //generate the pos_id from the new chracter
                let response: PrevNextCharacter =
                    find_prev_next(response_json.curr_row, response_json.curr_col, locked_data.clone(),response_json.curr_character);
                let mut position_id_prev: Vec<Id> = Vec::new();
                if response.row_prev != usize::MAX && response.col_prev != usize::MAX {
                    position_id_prev = locked_data[response.row_prev][response.col_prev].pos_id.clone();
                }
                let mut position_id_next: Vec<Id> = Vec::new();
                if response.row_next != usize::MAX && response.col_next != usize::MAX {
                    position_id_next = locked_data[response.row_next][response.col_next].pos_id.clone();
                }
                let position_identifier =
                    generate_pos_id(position_id_prev.clone(), position_id_next.clone(), user_id);
                let mut crdt_file_copy:Vec<Character> = locked_data.clone().into_iter().flatten().collect();
                let new_character = Character {
                    value: response.curr_character,
                    pos_id: position_identifier,
                    action_id: 0,
                };


                crdt_file_copy.push(new_character.clone());
                crdt_file_copy.sort_by(comp_character);

                //printing_elements
                if user_id == 1 {
                    for i in crdt_file_copy.clone() {
                        bering(format!("\n -> character : {}",i.value.to_string().as_str()), "position_id.txt");
                        for j in i.pos_id {
                            bering(format!(" [ {} : {} ] ", j.number, j.user), "position_id.txt");
                        }
                    }
                }
                //printing_elements

                let mut content = String::new();
                for i in &*crdt_file_copy {
                    content.push(i.value);
                }
                write_file(&content, file_path);
                if response.row_prev == usize::MAX {
                    locked_data[0].insert(0,new_character);
                }
                else {
                    locked_data[response.row_prev].insert(response.col_prev,new_character);
                }
            }
            _ => {}
        }
    }
}
