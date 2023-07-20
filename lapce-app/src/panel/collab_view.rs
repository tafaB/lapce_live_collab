use std::{fs::OpenOptions, path::Path, sync::Arc, thread};

use floem::{
    peniko::kurbo::Rect,
    reactive::{
        create_memo, create_rw_signal, create_signal, SignalGet, SignalSet,
        SignalUpdate,
    },
    style::{CursorStyle, Style},
    view::View,
    views::{
        container, label, list, scroll, stack, svg, virtual_list, Decorators,
        VirtualListDirection, VirtualListItemSize,
    },
    ViewContext,
};
use futures::{stream::SplitStream, SinkExt, StreamExt};
use lapce_proxy::buffer::Buffer;
use lapce_rpc::{buffer::BufferId, source_control::FileDiff};
use lapce_xi_rope::{Delta, Interval, Rope, RopeDelta};
use serde_json::json;
use url::Url;

use crate::{
    collab::{collab_session_mainloop, generate_initial_crdt},
    command::InternalCommand,
    command::LapceCommand,
    config::{color::LapceColor, icon::LapceIcons},
    editor::EditorData,
    id::EditorId,
    keypress::KeyPressFocus,
    settings::checkbox,
    source_control::SourceControlData,
    text_area::text_area,
    window,
    window_tab::{self, WindowTabData},
};

use super::{position::PanelPosition, view::panel_header};

use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use tokio::runtime::Runtime;
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::{CloseFrame, Message},
    WebSocketStream,
};

#[derive(Clone, Deserialize, Serialize)]
struct AppData {
    id: String,
    file_name: String,
    file_content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CollaborativeEditingData {
    pub file_name: String,
    pub file_content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StartResponse {
    pub status: String,
    pub id: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JoinResponse {
    pub status: String,
    pub file_content: String,
}

fn write_file(x: &str, y: &str) {
    let mut file = File::create(y).unwrap();
    file.write_all(x.as_bytes())
        .expect("Failed to write to file");
    file.flush().expect("Failed to flush file");
}

pub fn collab_panel(
    window_tab_data: Arc<WindowTabData>,
    _position: PanelPosition,
) -> impl View {
    let config = window_tab_data.common.config;
    let cx = ViewContext::get_current();
    let editor = EditorData::new_local(
        cx.scope,
        EditorId::next(),
        window_tab_data.common.clone(),
    );
    let cx_start = ViewContext::get_current();
    let editor_start = EditorData::new_local(
        cx_start.scope,
        EditorId::next(),
        window_tab_data.common.clone(),
    );

    stack(move || {
        (
            //START BUTTON
            stack(|| {
                let editor_clone = editor_start.clone();
                (
                    text_area(editor_clone)
                        .style(|| Style::BASE.width_pct(100.0).height_px(120.0)),
                    label(|| "START".to_string())
                        .style(move || {
                            Style::BASE
                                .margin_top_px(10.0)
                                .line_height(1.6)
                                .width_pct(100.0)
                                .justify_center()
                                .border(1.0)
                                .border_radius(6.0)
                                .border_color(
                                    *config
                                        .get()
                                        .get_color(LapceColor::LAPCE_BORDER),
                                )
                        })
                        .on_click(move |_| {
                            // on_click();
                            let editor_copy = editor_start.clone();
                            let input_text =
                                editor_copy.doc.get().buffer().to_string(); //the text from text_area;

                            let mut mut_app_data = AppData {
                                id: String::new(),
                                file_content: String::new(),
                                file_name: String::new(),
                            };
                            let path = Path::new(&input_text);
                            mut_app_data.file_name = path
                                .file_name()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string();
                            let mut file =
                                File::open(path).expect("Failed to open file");
                            file.read_to_string(&mut mut_app_data.file_content)
                                .expect("Failed to read file");
                            let rt = Runtime::new()
                                .expect("Failed to create Tokio runtime");
                            rt.block_on(async move {
                                let url = Url::parse("ws://localhost:8080").unwrap();
                                let (mut ws_stream, _response) = connect_async(url)
                                    .await
                                    .expect("Failed to connect");
                                let (mut write, mut read) = ws_stream.split();
                                let payload = json!({
                                    "action": "start_collab",
                                    "file_name": mut_app_data.file_name,
                                    "file_content": mut_app_data.file_content
                                });
                                let message = Message::Text(
                                    serde_json::to_string(&payload).unwrap(),
                                );
                                write
                                    .send(message)
                                    .await
                                    .expect("Failed to send data");
                                let message = read.next().await.expect("msg");
                                let message = message.unwrap();
                                match message {
                                    Message::Text(text) => {
                                        let response: StartResponse =
                                            serde_json::from_str(&text)
                                                .expect("Failed to parse response");
                                        let mut ctx: ClipboardContext =
                                            ClipboardProvider::new().expect(
                                                "Failed to initialize clipboard",
                                            );
                                        ctx.set_contents(
                                            response.id.to_string().to_owned(),
                                        )
                                        .expect("Failed to set clipboard contents");
                                        thread::spawn(move || {
                                            let rt = tokio::runtime::Runtime::new().unwrap();
                                            rt.block_on(async move {
                                                collab_session_mainloop(
                                                    &input_text,
                                                    generate_initial_crdt(mut_app_data.file_content, 1),
                                                    1
                                                )
                                                .await;
                                            });
                                        });
                                    }
                                    _ => {
                                        // error in message
                                    }
                                }
                            });

                            true
                        })
                        .hover_style(move || {
                            Style::BASE.cursor(CursorStyle::Pointer).background(
                                *config
                                    .get()
                                    .get_color(LapceColor::PANEL_HOVERED_BACKGROUND),
                            )
                        })
                        .active_style(move || {
                            Style::BASE.background(*config.get().get_color(
                                LapceColor::PANEL_HOVERED_ACTIVE_BACKGROUND,
                            ))
                        }),
                )
            })
            .style(|| Style::BASE.flex_col().size_pct(100.0, 100.0)),
            //SEPARATOR
            container(|| {
                svg(|| {
                    "<svg><rect width='1' height='100%' fill='black'/></svg>"
                        .to_string()
                })
                .style(|| Style::BASE.width_px(1.0).height_pct(100.0))
            }),
            //JOIN BUTTON
            stack(|| {
                let editor_clone = editor.clone();
                (
                    text_area(editor_clone)
                        .style(|| Style::BASE.width_pct(100.0).height_px(120.0)),
                    label(|| "JOIN".to_string())
                        .style(move || {
                            Style::BASE
                                .margin_top_px(10.0)
                                .line_height(1.6)
                                .width_pct(100.0)
                                .justify_center()
                                .border(1.0)
                                .border_radius(6.0)
                                .border_color(
                                    *config
                                        .get()
                                        .get_color(LapceColor::LAPCE_BORDER),
                                )
                        })
                        .on_click(move |_| {
                            let editor_copy = editor.clone();
                            let mut input_text =
                                editor_copy.doc.get().buffer().to_string(); //the text from text_area;
                            let session_id:i32 = input_text.parse().expect("error parsing");
                            let rt = Runtime::new()
                                .expect("Failed to create Tokio runtime");
                            rt.block_on(async move {
                                let url = Url::parse("ws://localhost:8080").unwrap();
                                let (ws_stream, _response) = connect_async(url)
                                    .await
                                    .expect("Failed to connect");
                                let (mut write, mut read) = ws_stream.split();
                                //   {
                                //     "action" : "join_collab",
                                //     "session_id" : 7
                                //   }

                                let payload = json!({
                                    "action": "join_collab",
                                    "session_id": session_id
                                });
                                let message = Message::Text(
                                    serde_json::to_string(&payload).unwrap(),
                                );
                                write
                                    .send(message)
                                    .await
                                    .expect("Failed to send data");
                                let message = read.next().await.expect("msg");
                                let message = message.unwrap();
                                match message {
                                    Message::Text(text) => {
                                        let response: JoinResponse =
                                            serde_json::from_str(&text)
                                                .expect("Failed to parse response");
                                        write_file(response.file_content.as_str(), "bering_lapce_collab.txt");
                                        thread::spawn(move || {
                                            let rt = tokio::runtime::Runtime::new().unwrap();
                                            rt.block_on(async move {
                                                collab_session_mainloop(
                                                    &String::from("/Users/beringtafa/lapce/bering_lapce_collab.txt"),
                                                    generate_initial_crdt(response.file_content, 1),
                                                    2,
                                                )
                                                .await;
                                            });
                                        });
                                    }
                                    _ => {
                                        // error in message
                                    }
                                }
                            });
                            true
                        })
                        .hover_style(move || {
                            Style::BASE.cursor(CursorStyle::Pointer).background(
                                *config
                                    .get()
                                    .get_color(LapceColor::PANEL_HOVERED_BACKGROUND),
                            )
                        })
                        .active_style(move || {
                            Style::BASE.background(*config.get().get_color(
                                LapceColor::PANEL_HOVERED_ACTIVE_BACKGROUND,
                            ))
                        }),
                )
            })
            .style(|| Style::BASE.flex_col().width_pct(100.0).padding_px(10.0)),
        )
    })
    .style(|| Style::BASE.flex_col().size_pct(100.0, 100.0))
}
