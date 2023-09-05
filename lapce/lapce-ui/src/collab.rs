use std::sync::Arc;

use druid::{
    kurbo::BezPath,
    piet::{Text, TextLayout as PietTextLayout, TextLayoutBuilder},
    BoxConstraints, Command, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, MouseButton, MouseEvent, PaintCtx, Point, Rect, RenderContext,
    Size, Target, UpdateCtx, Widget, WidgetExt, WidgetId,
};
use lapce_data::{
    command::{
        CommandKind, LapceCommand, LapceUICommand, LapceWorkbenchCommand,
        LAPCE_COMMAND, LAPCE_UI_COMMAND,
    },
    config::{LapceIcons, LapceTheme},
    data::{FocusArea, LapceData, LapceTabData},
    panel::PanelKind,
};
use lapce_rpc::source_control::FileDiff;

use crate::{
    button::Button,
    editor::view::LapceEditorView,
    panel::{LapcePanel, PanelHeaderKind, PanelSizing},
};

use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use std::fs::File;
use std::io::prelude::*;

#[derive(Clone, Deserialize, Serialize)]
struct AppData {
    id: String,
    file_name: String,
    file_content: String,
}

pub fn new_collab_panel(data: &LapceTabData) -> LapcePanel {
    let file_name = data.log_file.as_ref().clone().unwrap();
    let mut file = File::create("tafa.txt").expect("Failed to create file");
    let buffer = String::new();
    file.write_all(buffer.as_bytes())
        .expect("Failed to write to file");
    let editor_data = data
        .main_split
        .editors
        .get(&data.source_control.editor_view_id)
        .unwrap();
    let input =
        LapceEditorView::new(editor_data.view_id, editor_data.editor_id, None)
            .hide_header()
            .hide_gutter()
            .set_placeholder("Add the link from your partner ...".to_string())
            .padding((15.0, 15.0));

    let app_data = AppData {
        id: "123".to_string(),
        file_name: "example.txt".to_string(),
        file_content: "Hello, world!".to_string(),
    };

    let start_button = Button::new(data, "START")
        .on_click(move |_, _, _| {
            println!("Hello World!");
            let client = Client::new();
            let rt = Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async {
                let json_body = serde_json::to_string(&app_data)
                    .expect("Failed to serialize JSON");

                let response = client
                    .post("http://localhost:8000/api/start_collab")
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(json_body)
                    .send()
                    .await;

                match response {
                    Ok(res) => {
                        if res.status().is_success() {
                            let json_response =
                                res.json::<serde_json::Value>().await;
                            match json_response {
                                Ok(json) => {
                                    let temp = json.to_string();
                                    let mut temp = temp.chars();
                                    temp.next();
                                    temp.next_back();
                                    let response_str = temp.as_str().to_string();
                                    //other way : response_str = response_str.split_at(response_str.len()-1).0.split_at(1).1.to_string();
                                    println!("Response: {}", response_str);
                                    let mut ctx: ClipboardContext =
                                        ClipboardProvider::new().unwrap();
                                    ctx.set_contents(response_str).unwrap();
                                }
                                Err(err) => println!(
                                    "Failed to parse response as JSON: {}",
                                    err.to_string()
                                ),
                            }
                        } else {
                            println!(
                                "POST request failed with status: {}",
                                res.status()
                            );
                        }
                    }
                    Err(err) => {
                        println!("Failed to send request: {}", err.to_string());
                    }
                }
            });
        })
        .expand_width()
        .with_id(data.source_control.commit_button_id)
        .padding((10.0, 0.0, 10.0, 10.0));

    let join_button = Button::new(data, "JOIN")
        .on_click(|ctx, data, _env| {
            ctx.submit_command(Command::new(
                LAPCE_COMMAND,
                LapceCommand {
                    kind: CommandKind::Workbench(
                        LapceWorkbenchCommand::SourceControlCommit,
                    ),
                    data: None,
                },
                Target::Widget(data.id),
            ));
        })
        .expand_width()
        .with_id(data.source_control.commit_button_id)
        .padding((10.0, 0.0, 10.0, 10.0));

    LapcePanel::new(
        PanelKind::Collab,
        data.source_control.widget_id,
        data.source_control.split_id,
        vec![
            (
                data.source_control.commit_button_id,
                PanelHeaderKind::None,
                start_button.boxed(),
                PanelSizing::Flex(false),
            ),
            (
                editor_data.view_id,
                PanelHeaderKind::None,
                input.boxed(),
                PanelSizing::Size(300.0),
            ),
            (
                data.source_control.commit_button_id,
                PanelHeaderKind::None,
                join_button.boxed(),
                PanelSizing::Flex(false),
            ),
        ],
    )
}
