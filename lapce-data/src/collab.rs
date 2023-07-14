use std::path::PathBuf;

use druid::{Command, Env, EventCtx, Modifiers, Target, WidgetId};
use indexmap::IndexMap;
use lapce_core::{
    command::{FocusCommand, MoveCommand},
    mode::Mode,
    movement::Movement,
};
use lapce_rpc::source_control::FileDiff;

use crate::{
    command::{CommandExecuted, CommandKind, LapceUICommand, LAPCE_UI_COMMAND},
    keypress::KeyPressFocus,
    split::{SplitDirection, SplitMoveDirection},
};

pub const SOURCE_CONTROL_BUFFER: &str = "[Source Control Buffer]";
pub const SEARCH_BUFFER: &str = "[Search Buffer]";

#[derive(Clone)]
pub struct  CollabData{
    pub active: WidgetId,
    pub widget_id: WidgetId,
    pub split_id: WidgetId,
    pub split_direction: SplitDirection,
    pub editor_view_id: WidgetId,
    pub commit_button_id: WidgetId,
}

impl CollabData {
    pub fn new() -> Self {
        let editor_view_id = WidgetId::next();
        Self {
            active: editor_view_id,
            widget_id: WidgetId::next(),
            editor_view_id,
            commit_button_id: WidgetId::next(),
            split_id: WidgetId::next(),
            split_direction: SplitDirection::Horizontal,
        }
    }
}

impl Default for CollabData {
    fn default() -> Self {
        Self::new()
    }
}