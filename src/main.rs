#![allow(unused)]

mod config;
mod document;
mod editor;
mod filetype;
mod highlighting;
mod row;
mod terminal;
mod utils;

use std::io;
use editor::Editor;
use std::env;


fn main () -> Result<(), io::Error> {
    let mut editor = Editor::default();

    editor.run()?;
    Ok(())
}
