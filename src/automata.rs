use std::collections::HashSet;
use crate::editor::Editor;
use crate::utils::{v_jump_to_line, Position, PromptCallbackCode, Promptable, ScrollDirection};
use termion::event::Key;
use crate::automata::commands::to_next_word_start;
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
    Delete,
    Change,
    Yank,
    Togglecase,
    Lowercase,
    Uppercase,
    G,
    Z
}

pub struct EditorFSM {
    pub state:              EditorState,
    pub command_buffer:     String,
    pub last_command:       String,
    pub command_count:      usize,
    pub nav_object_count:   usize,
}

impl EditorFSM {
    pub fn new() -> Self {
        EditorFSM {
            state: EditorState::Normal,
            command_buffer: String::new(),
            last_command: String::new(),
            command_count: 0,
            nav_object_count: 0,
        }
    }
    pub fn success_log(&mut self) { log!("done executing: {}", self.command_buffer); }

    pub fn success_exit(&mut self) {
        self.command_count = 0;
        self.nav_object_count = 0;
        self.state = EditorState::Normal;
        self.success_log();
        self.command_buffer.clear();
    }

    pub fn run(&mut self, base_key: &char, editor: &mut Editor) {
        use self::commands::{move_right, move_down, move_left,
                             move_up, to_last_line, to_line_start,
                             to_top_screen, to_bottom_screen, to_mid_screen,
                             to_line_end};

        // STATE MACHINE FOR INPUT HANDLING
        match *base_key {
            'l' => {
                move_right(self, editor, 1);
                return;
            },
            'j' => {
                move_down(self, editor, 1);
                return;
            },
            'h' => {
                move_left(self, editor, 1);
                return;
            },
            'w' => {
                to_next_word_start(self, editor,  1);
                return;
            },
            'k' => {
                move_up(self, editor, 1);
                return;
            },
            'G' => {
                to_last_line(self, editor, 1);
                return;

            },
            '0' => {
                to_line_start(self, editor, 1);
                return;
            },
            'H' => {
                to_top_screen(self, editor, 1);
                return;
            },
            'L' => {
                to_bottom_screen(self, editor, 1);
                return;
            },
            'M' => {
                to_mid_screen(self, editor, 1);
                return;
            },
            '$' => {
                to_line_end(self, editor, 1);
                return;
            },
            _ => ()
        }


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
                        v_jump_to_line(editor, fsm, &'g');
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
                        v_jump_to_line(editor, fsm, &'G');
                        fsm.success_exit();

                        return PromptCallbackCode::Success;
                    }
                    return PromptCallbackCode::Continue;
                },
                Key::Char('~') => {
                    if fsm.state == EditorState::G {
                        fsm.state = EditorState::Togglecase;
                        fsm.command_buffer.pop();
                        fsm.command_buffer.push('~');

                        return PromptCallbackCode::Continue;
                    }
                    return PromptCallbackCode::Failure;
                },
                Key::Char('0'..='9') => match key {
                    Key::Char(key) => {
                        let number = key.to_digit(10).expect("failed to parse action key!");
                        if fsm.state == EditorState::Normal { // we are just starting the command or we're still racking up the command_count
                            fsm.command_count = (fsm.command_count * 10).saturating_add(number as usize);
                        } else {
                            match fsm.state {
                                EditorState::Change | EditorState::Delete | EditorState::Yank | EditorState::Visual | EditorState::Lowercase | EditorState::Uppercase | EditorState::Togglecase => { // only allow additional prefix collection for navigation objects
                                    fsm.nav_object_count = (fsm.nav_object_count * 10).saturating_add(number as usize);
                                },
                                _ => {
                                    return PromptCallbackCode::Failure;
                                }
                            }
                        }

                        fsm.command_buffer.push(key);
                        return PromptCallbackCode::Continue;
                    },
                    _ => { panic!("{}", INVARIANT_ERROR_MESSAGE) }
                },
                Key::Char(x) => {
                    if fsm.state == EditorState::Normal {
                        match x {
                            'l' => {
                                move_right(fsm, editor, fsm.command_count);

                                fsm.command_buffer.push('l');
                                fsm.success_exit();
                                return PromptCallbackCode::Success;
                            },
                            'j' => {
                                move_down(fsm, editor,  fsm.command_count);

                                fsm.command_buffer.push('j');
                                fsm.success_exit();
                                return PromptCallbackCode::Success;
                            },
                            'h' => {
                                move_left(fsm, editor,  fsm.command_count);

                                fsm.command_buffer.push('h');
                                fsm.success_exit();
                                return PromptCallbackCode::Success;
                            },
                            'k' => {
                                move_up(fsm, editor,  fsm.command_count);

                                fsm.command_buffer.push('k');
                                fsm.success_exit();
                                return PromptCallbackCode::Success;
                            },
                            'G' => {
                                to_last_line(fsm, editor,  fsm.command_count);

                                fsm.command_buffer.push('G');
                                fsm.success_exit();
                                return PromptCallbackCode::Success;
                            },
                            '0' => {
                                to_line_start(fsm, editor,  fsm.command_count);

                                fsm.command_buffer.push('0');
                                fsm.success_exit();
                                return PromptCallbackCode::Success;
                            },
                            'H' => {
                                to_top_screen(fsm, editor,  fsm.command_count);

                                fsm.command_buffer.push('H');
                                fsm.success_exit();
                                return PromptCallbackCode::Success;
                            },
                            'L' => {
                                to_bottom_screen(fsm, editor,   fsm.command_count);

                                fsm.command_buffer.push('L');
                                fsm.success_exit();
                                return PromptCallbackCode::Success;
                            },
                            'M' => {
                                to_mid_screen(fsm, editor,  fsm.command_count);

                                fsm.command_buffer.push('M');
                                fsm.success_exit();
                                return PromptCallbackCode::Success;
                            },
                            '$' => {
                                to_line_end(fsm, editor,   fsm.command_count);

                                fsm.command_buffer.push('$');
                                fsm.success_exit();
                                return PromptCallbackCode::Success;
                            },
                            _ => ()
                        }
                    }
                    return PromptCallbackCode::Continue;
                },
                _ => { return PromptCallbackCode::Failure; }
            }

        }, None);
    }
}

impl Promptable for EditorFSM { }


pub mod commands {
    use std::collections::HashMap;
    use crate::editor::Editor;
    use crate::EditorFSM;
    use crate::utils::{find_char_position, find_string_position, get_v_char_class, is_word, ScrollDirection, VCharacterClass};
    use unicode_segmentation::UnicodeSegmentation;
    use crate::log;

    pub fn move_right (fsm: &mut EditorFSM, editor: &mut Editor, action_count: usize) {
        if let Some(curr_row) = editor.document.rows.get(editor.cursor_position.y as usize)
        {
            if let Some(curr_row) = editor.document.rows.get(editor.cursor_position.y as usize)
            {
                if action_count > 1 {
                    let mut mag = 1_usize.saturating_mul(action_count);

                    if editor.cursor_position.x.saturating_add(mag as u16) as usize >= curr_row.len {
                        editor.cursor_position.x = curr_row.len.saturating_sub(1) as u16;
                    } else {
                        editor.cursor_position.x = editor.cursor_position.x.saturating_add(mag as u16);
                    }
                } else if editor.cursor_position.x == curr_row.len.saturating_sub(1) as u16 {
                    if let Some(next_row) = editor
                        .document
                        .rows
                        .get(editor.cursor_position.y.saturating_add(1) as usize)
                    {
                        editor.cursor_position.y = editor.cursor_position.y.saturating_add(1);
                        editor.cursor_position.x = 0;
                    }
                } else {
                    editor.cursor_position.x = editor.cursor_position.x.saturating_add(1);
                }
                    editor.movement_data.last_nav_position.x = editor.cursor_position.x;
            }
        }

        return;
    }

    pub fn move_down (fsm: &mut EditorFSM, editor: &mut Editor, action_count: usize) {
        if editor.cursor_position.y == editor.document.rows.len().saturating_sub(1) as u16 {
            fsm.success_exit();
            return;
        }

        let mag = 1_usize.saturating_mul(action_count);

        if let Some(next_row) = editor
            .document
            .rows
            .get(editor.cursor_position.y.saturating_add(mag as u16) as usize)
        {

            if next_row.len <= editor.movement_data.last_nav_position.x as usize {
                editor.cursor_position.x = next_row.len.saturating_sub(1) as u16;
            } else {
                editor.cursor_position.x = editor.movement_data.last_nav_position.x;
            }
            editor.cursor_position.y = editor.cursor_position.y.saturating_add(mag as u16);
        }

        editor.update_selection();
        editor.scroll(ScrollDirection::Down);

        return;
    }

    pub fn move_left (fsm: &mut EditorFSM, editor: &mut Editor, action_count: usize) {
        if let Some(curr_row) = editor.document.rows.get(editor.cursor_position.y as usize)
        {
            if action_count > 1 {
                let mut mag = 1_usize.saturating_mul(action_count);

                if editor.cursor_position.x.saturating_sub(mag as u16) as usize <= 0 {
                    editor.cursor_position.x = 0;
                } else {
                    editor.cursor_position.x = editor.cursor_position.x.saturating_sub(mag as u16);
                }
            } else if editor.cursor_position.x == 0_u16 {
                if (editor.cursor_position.y == 0_u16) {
                    fsm.success_exit();
                    return;
                }
                if let Some(prev_row) = editor
                    .document
                    .rows
                    .get(editor.cursor_position.y.saturating_sub(1) as usize)
                {
                    editor.cursor_position.y = editor.cursor_position.y.saturating_sub(1);
                    editor.cursor_position.x = prev_row.len.saturating_sub(1) as u16;
                }
            } else {
                editor.cursor_position.x = editor.cursor_position.x.saturating_sub(1);
            }

            editor.movement_data.last_nav_position.x = editor.cursor_position.x;
        }

        return;
    }

    pub fn move_up (fsm: &mut EditorFSM, editor: &mut Editor, action_count: usize) {
        if editor.cursor_position.y == 0 {
            fsm.success_exit();
            return;
        }

        let mag = 1_usize.saturating_mul(action_count);

        if let Some(prev_row) = editor
            .document
            .rows
            .get(editor.cursor_position.y.saturating_sub(mag as u16) as usize)
        {
            if prev_row.len <= editor.movement_data.last_nav_position.x as usize {
                editor.cursor_position.x = prev_row.len.saturating_sub(1) as u16;
            } else {
                editor.cursor_position.x = editor.movement_data.last_nav_position.x;
            }
            editor.cursor_position.y = editor.cursor_position.y.saturating_sub(mag as u16);
        }

        return;
    }

    pub fn to_last_line (fsm: &mut EditorFSM, editor: &mut Editor, action_count: usize) {
        if let Some(curr_row) = editor
            .document
            .rows
            .get(editor.document.rows.len().saturating_sub(1) as usize)
        {
            editor.cursor_position.y = editor.document.rows.len().saturating_sub(1) as u16;
            if curr_row.len <= editor.movement_data.last_nav_position.x as usize {
                editor.cursor_position.x = curr_row.len.saturating_sub(1) as u16;
            } else {
                editor.cursor_position.x = editor.movement_data.last_nav_position.x;
            }
        }

        return;
    }

    pub fn to_line_start (fsm: &mut EditorFSM, editor: &mut Editor, action_count: usize) {
        editor.cursor_position.x = 0;
        editor.movement_data.last_nav_position.x = editor.cursor_position.x;

        return;
    }

    pub fn to_top_screen (fsm: &mut EditorFSM, editor: &mut Editor, action_count: usize) {
        if let Some(curr_row) = editor.document.rows.get(editor.offset.y as usize)
        {
            if curr_row.len < (editor.movement_data.last_nav_position.x) as usize {
                editor.cursor_position.x = curr_row.len.saturating_sub(1) as u16;
            } else {
                editor.cursor_position.x = editor.movement_data.last_nav_position.x;
            }
            editor.cursor_position.y = editor.offset.y;
        }

        return;
    }

    pub fn to_bottom_screen (fsm: &mut EditorFSM, editor: &mut Editor, action_count: usize) {
        let height = editor.net_height;
        let bottom = editor.offset.y.saturating_add(height).saturating_add(1);

        if let Some(curr_row) = editor.document.rows.get(bottom as usize)
        {
            if curr_row.len < (editor.movement_data.last_nav_position.x) as usize {
                editor.cursor_position.x = curr_row.len.saturating_sub(1) as u16;
            } else {
                editor.cursor_position.x = editor.movement_data.last_nav_position.x;
            }

            editor.cursor_position.y = bottom;
        }

        return;
    }

    pub fn to_mid_screen (fsm: &mut EditorFSM, editor: &mut Editor, action_count: usize) {
        let height = editor.net_height;
        let middle_position = (editor.offset.y.saturating_add(height.saturating_div(2)));

        if let Some(curr_row) = editor.document.rows.get(middle_position as usize)
        {
            if curr_row.len < (editor.movement_data.last_nav_position.x) as usize {
                editor.cursor_position.x = curr_row.len.saturating_sub(1) as u16;
            } else {
                editor.cursor_position.x = editor.movement_data.last_nav_position.x;
            }
            editor.cursor_position.y = middle_position;
        }

        return;
    }

    pub fn to_line_end (fsm: &mut EditorFSM, editor: &mut Editor, action_count: usize) {
        if let Some(curr_row) = editor.document.rows.get(editor.cursor_position.y as usize)
        {
            editor.cursor_position.x = curr_row.len.saturating_sub(1) as u16;
            editor.movement_data.last_nav_position.x = editor.cursor_position.x;
        }

        return;
    }

    pub fn to_next_word_start (fsm: &mut EditorFSM, editor: &mut Editor, action_count: usize) { // well, technically it's more than just word start
        if let Some(curr_row) = editor.document.rows.get(editor.cursor_position.y as usize)
        {
            let mut graphemes = curr_row.string.graphemes(true).collect::<Vec<&str>>();
            let mut next_match_idx: i32 = 0;
            let mut index = editor.cursor_position.x;
            let mut flags = (false/*word start*/, false/*blank start*/, false/* graph start*/);
            let mut search_flags = (false/*word -> blank*/, false/*blank -> graph*/, false/*graph1 -> graph2*/);
            let mut x = index;
            let mut class_hash: HashMap<String, i32> = HashMap::new();

            next_match_idx = loop {
                if let Some(grapheme) = graphemes.get(x as usize) {
                    let at_start = x == editor.cursor_position.x;
                    match (*grapheme).chars().next() {// TODO: graph1 -> graph2 transitions jump character classes even tho they are separated by a space
                        Some(c) => {
                            if is_word(c) && at_start { /* cursor on a word character or blank*/  flags.0 = true;}
                            else if c.is_whitespace() && at_start { flags.1 = true; }
                            else if c.is_ascii_graphic() && at_start { flags.2 = true; }

                            if is_word(c) && at_start && flags.0 { search_flags.0 = true; }/*word -> blank*/
                            else if c.is_whitespace() && at_start && flags.1 { search_flags.1 = true; }/*blank to graph*/
                            else if c.is_ascii_graphic() && at_start && flags.2 { search_flags.2 = true; }/*graph1 -> graph2*/

                            if flags.0 && is_word(c) && search_flags.0 && !(search_flags.1 || search_flags.2) {
                                search_flags.0 = true;
                            } else if flags.0 && (c.is_whitespace() || (c.is_ascii_graphic() && !is_word(c))) && search_flags.0 && !(search_flags.1 || search_flags.2) {
                                search_flags.0 = false;
                                search_flags.1 = true;
                                if !c.is_whitespace() { break x as i32; }
                            } else if flags.0 && c.is_ascii_graphic() && search_flags.1 && !(search_flags.0 || search_flags.2) {
                                break x as i32;
                            }

                            if flags.1 && c.is_whitespace() && search_flags.1 && !(search_flags.0 || search_flags.2) {
                                search_flags.1 = true;
                            } else if flags.1 && c.is_ascii_graphic() && search_flags.1 && !(search_flags.0 || search_flags.2) {
                                break x as i32;
                            }

                            if flags.2 && c.is_ascii_graphic() && !is_word(c) && search_flags.2 && !(search_flags.0 || search_flags.1) {
                                search_flags.2 = true;
                                let class: String = get_v_char_class(c).into();
                                if let Some(c) = class_hash.get_mut(&class) {
                                    *c += 1;
                                } else { class_hash.insert(class, 1); }
                            } else if flags.2 && c.is_ascii_graphic() && search_flags.2 && !(search_flags.0 || search_flags.1) {
                                let class: String = get_v_char_class(c).into();
                                if !class_hash.contains_key(&class) {
                                    break x as i32;
                                } else {
                                    if let Some(c) = class_hash.get_mut(&class) { *c += 1; }
                                }
                            }

                        },
                        _ => { return; }
                    }
                }
                x += 1;

                if (x as usize) >= graphemes.len() { break -1_i32; }
            };

            if next_match_idx <= index as i32 { return; }

            editor.cursor_position.x = next_match_idx as u16;
            editor.movement_data.last_nav_position.x = editor.cursor_position.x;
        }

    }
}