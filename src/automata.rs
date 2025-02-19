use std::collections::HashSet;
use crate::editor::Editor;
use crate::utils::{jump_to_line, Position, PromptCallbackCode, Promptable};
use termion::event::Key;
use crate::config::INVARIANT_ERROR_MESSAGE;
use crate::terminal::Terminal;
use crate::log;

#[derive(Debug, PartialEq)]
pub enum EditorState {
    Visual,
    MLVisual,
    BlockVisual,
    Normal,
    Replace,
    Search,
    LineScan,
    G,
    Z,
    Delete,
    Change
}

pub struct EditorFSM {
    pub state:              EditorState,
    pub command_buffer:     String,
    pub last_command:       String,
    pub command_count:      usize,
}

impl EditorFSM {
    pub fn new() -> Self {
        EditorFSM {
            state: EditorState::Normal,
            command_buffer: String::new(),
            last_command: String::new(),
            command_count: 0,
        }
    }
    pub fn success_log(&mut self) {
        log!("done executing: {}", self.command_buffer);
    }

    pub fn success_exit(&mut self) {
        self.command_count = 0;
        self.state = EditorState::Normal;
        self.success_log();
        self.command_buffer.clear();
    }

    pub fn run(&mut self, base_key: &char, editor: &mut Editor) {

        // STATE MACHINE FOR EDITOR STATE
        match base_key {
            'v' | 'V' => {
                if *base_key == 'V' {
                    self.state = EditorState::MLVisual;
                    self.command_buffer.push(*base_key);
                } else {
                    self.state = EditorState::Visual;
                    self.command_buffer.push(*base_key);
                }
            },
            'r' => {
                self.state = EditorState::Replace;
                self.command_buffer.push(*base_key);
            },
            '/' => {
                self.state = EditorState::Search;
                self.command_buffer.push(*base_key);
            },
            'f' => {
                self.state = EditorState::LineScan;
                self.command_buffer.push(*base_key);
            },
            'g' => {
                self.state = EditorState::G;
                self.command_buffer.push(*base_key);
            },
            'z' => {
                self.state = EditorState::Z;
                self.command_buffer.push(*base_key);
            },
            'c' => {
                self.state = EditorState::Change;
                self.command_buffer.push(*base_key);
            },
            'd' => {
                self.state = EditorState::Delete;
                self.command_buffer.push(*base_key);
            },
            '0'..='9' => {
                let number = base_key.to_digit(10).expect("failed to parse base key!");
                self.command_count = (self.command_count * 10).saturating_add(number as usize);
                self.command_buffer.push(*base_key);
            },
            _ => ()
        }

        self.prompt_exec( |fsm, key| {
            match key {
                Key::Char('g') => {
                    if fsm.state == EditorState::G {
                        jump_to_line(editor, fsm, &'g');
                        fsm.success_exit();

                        return PromptCallbackCode::Success;
                    } else if fsm.state == EditorState::Normal {
                        fsm.state = EditorState::G;
                        fsm.command_buffer.push('g');
                    }
                    return PromptCallbackCode::Continue;
                },
                Key::Char('G') => {
                    if fsm.state == EditorState::Normal {
                        jump_to_line(editor, fsm, &'G');
                        fsm.success_exit();

                        return PromptCallbackCode::Success;
                    }
                    return PromptCallbackCode::Continue;
                },
                Key::Char('0'..='9') => match key {
                    Key::Char(key) => {
                        let number = key.to_digit(10).expect("failed to parse action key!");
                        if fsm.state == EditorState::Normal { // we are just starting the command or we're still racking up the command_count
                            fsm.command_count = (fsm.command_count * 10).saturating_add(number as usize);
                        }

                        fsm.command_buffer.push(key);
                        return PromptCallbackCode::Continue;
                    },
                    _ => { panic!("{}", INVARIANT_ERROR_MESSAGE) }
                }

                ,
                _ => { return PromptCallbackCode::Continue; }
            }

        }, None);
    }
}

impl Promptable for EditorFSM { }