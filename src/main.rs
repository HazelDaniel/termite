#![allow(unused)]

mod automata;
mod config;
mod document;
mod editor;
mod filetype;
mod highlighting;
mod row;
mod terminal;
mod utils;


use editor::Editor;
use std::env;
use std::io;
use once_cell::sync::OnceCell;
use tokio;
use crate::automata::EditorFSM;
use crate::utils::{OrderedLogger, LOGGER};


#[tokio::main]
async fn main() -> Result<(), io::Error> {
    #![allow(non_snake_case)]
    let mut FSM: EditorFSM = EditorFSM::new();

    let args: Vec<String> = env::args().collect();
    let mut log_flag_idx = 0_i16;
    let arg_size = args.len();
    let log_file_index: i16 = loop {
        if (log_flag_idx as usize) >= arg_size.saturating_sub(1) {break -1};
        if args[log_flag_idx as usize] == "--log" {
            break if (log_flag_idx as usize) < arg_size.saturating_sub(1) {log_flag_idx} else {-1};
        }
        log_flag_idx += 1;
    };

    let mut editor = Editor::default();

    if log_file_index != -1 {
        let log_file = &args[log_file_index.saturating_add(1) as usize];
        LOGGER.set(OrderedLogger::new(&log_file).unwrap());
    };


    editor.run(&mut FSM).await?;
    Ok(())
}
