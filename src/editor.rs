use std::cell::RefCell;
use std::future::Future;
use std::io;
use std::io::{stdin, stdout, ErrorKind, Read, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use chrono::{Local, Timelike};
use unicode_segmentation::UnicodeSegmentation;
use std::rc::{Rc};
use std::sync::{Arc, Mutex, MutexGuard};
use once_cell::sync::OnceCell;
use crate::automata::EditorFSM;
use crate::config::{DEFAULT_QUIT_TIMES, EDITOR_NAME, PACKAGE_VERSION};
use crate::document::Document;
use crate::log;
use crate::row::Row;
use crate::terminal::Terminal;
use crate::utils::{die, HighlightingOptions, MovementData, Position, Size, StatusMessage, TerminalMode, ScrollDirection, Selection, Promptable};

pub struct Editor {
    pub should_quit:                bool,
    pub terminal:                   Terminal,
    pub cursor_position:            Position,
    pub offset:                     Position,
    pub document:                   Document,
    pub status_message:             Option<StatusMessage>,
    pub quit_times:                 u8,
    pub highlighted_word:           Option<String>,
    pub mode:                       TerminalMode,
    pub movement_data:              MovementData,
    pub selection:                  Option<Selection>,
    pub net_height:                 u16
}


impl Default for Editor {
    fn default() -> Editor {

        Self {
            offset: Position::default(),
            cursor_position: Position::default(),
            quit_times: DEFAULT_QUIT_TIMES,
            document: Document::default(),
            highlighted_word: None,
            terminal: Terminal::default(),
            should_quit: false,
            status_message: None,
            mode: TerminalMode::Normal,
            movement_data: MovementData::default(),
            selection: None,
            net_height: 0
        }
    }
}

impl Editor {
    pub async fn run(&mut self, fsm: &mut EditorFSM) -> Result<(), io::Error> {
        let mut terminal = self.terminal.get_std_buffer();
        let Size { width, height } = self.terminal.get_size();
        self.net_height = height;
        self.document.load();

        loop {
            match self.refresh_screen() {
                Ok(res) => {}
                Err(error) => die(error),
            };

            if (self.should_quit) {
                // print goodbye message and cleanup
                println!("goodbye!");
                return Ok(());
            }

            match self.process_keys(fsm).await {
                Ok(res) => {}
                Err(error) => die(error),
            }
        }
    }
    pub fn refresh_screen(&mut self) -> Result<(), io::Error> {
        let Position {
            x: offset_x,
            y: offset_y,
        } = self.offset;
        let Size { width, height } = self.terminal.get_size();
        self.move_cursor(Position::default());
        self.terminal.cursor_hide();
        if self.should_quit {
            self.terminal.clear_screen();
            self.terminal.cursor_show();
            return Ok(());
        }

        self.document.highlight(
            &HighlightingOptions::default(),
            &self.highlighted_word,
            Some(offset_y.saturating_add(height))
        );

        self.draw_rows();

        self.net_height = height;
        self.net_height = self.net_height.saturating_sub(self.draw_status_bar()?);
        self.net_height = self.net_height.saturating_sub(self.draw_message_bar(None)?);

        self.move_cursor(Position {
            x: self.cursor_position.x.saturating_sub(offset_x),
            y: self.cursor_position.y.saturating_sub(offset_y),
        });

        self.terminal.cursor_show();
        self.terminal.flush()?;

        Ok(())
    }

    pub fn display_welcome_message(&self) {
        let Size { width, height } = self.terminal.get_size();
        let welcome_message: String = "".to_owned() + EDITOR_NAME + ". v" + PACKAGE_VERSION;
        let message_len = welcome_message.len();
        let width_diff = width - message_len as u16;
        let pad_len = width_diff / 2;
        let mut l_pad = " ".repeat(pad_len as usize);
        l_pad.truncate(pad_len.saturating_sub(1) as usize);
        let r_pad = " ".repeat(pad_len as usize);
        print!("~{}{}{}\n\r", l_pad, welcome_message, r_pad);
    }

    pub fn move_cursor(&self, pos: Position) -> Result<(), io::Error> {
        let Position { x, y } = pos;

        self.terminal.goto(Position { x, y });

        Ok(())
    }

    pub async fn process_keys(&mut self, fsm: &mut EditorFSM) -> Result<(), io::Error> {
        if self.mode == TerminalMode::Normal {
            self.process_normal_mode(fsm).await?;
        }

        Ok(())
    }

    pub async fn process_normal_mode(&mut self, fsm: &mut EditorFSM) -> Result<(), io::Error> {
        let Size { height, .. } = self.terminal.get_size();

        if let Some(key) = stdin().keys().next() {
            match key? {
                Key::Char('\n') => {
                    print!("{}", "\n\r");
                },
                Key::Char('l') => {
                    if let Some(curr_row) = self.document.rows.get(self.cursor_position.y as usize)
                    {
                        if self.cursor_position.x == curr_row.len.saturating_sub(1) as u16 {
                            if let Some(next_row) = self
                                .document
                                .rows
                                .get(self.cursor_position.y.saturating_add(1) as usize)
                            {
                                self.cursor_position.y = self.cursor_position.y.saturating_add(1);
                                self.cursor_position.x = 0;
                            }
                        } else {
                            self.cursor_position.x = self.cursor_position.x.saturating_add(1);
                        }
                        self.movement_data.last_nav_position.x = self.cursor_position.x;
                    }
                },
                Key::Char('j') => {
                    if self.cursor_position.y == self.document.rows.len().saturating_sub(1) as u16 {
                        return Ok(());
                    }

                    if let Some(next_row) = self
                        .document
                        .rows
                        .get(self.cursor_position.y.saturating_add(1) as usize)
                    {
                        if next_row.len <= self.movement_data.last_nav_position.x as usize {
                            self.cursor_position.x = next_row.len.saturating_sub(1) as u16;
                        } else {
                            self.cursor_position.x = self.movement_data.last_nav_position.x;
                        }
                        self.cursor_position.y = self.cursor_position.y.saturating_add(1);
                    }

                    self.update_selection();
                    self.scroll(ScrollDirection::Down);
                    return Ok(())
                },
                Key::Char('h') => {
                    if let Some(curr_row) = self.document.rows.get(self.cursor_position.y as usize)
                    {
                        if self.cursor_position.x == 0_u16 {
                            if (self.cursor_position.y == 0_u16) {
                                return Ok(());
                            }
                            if let Some(prev_row) = self
                                .document
                                .rows
                                .get(self.cursor_position.y.saturating_sub(1) as usize)
                            {
                                self.cursor_position.y = self.cursor_position.y.saturating_sub(1);
                                self.cursor_position.x = prev_row.len.saturating_sub(1) as u16;
                            }
                        } else {
                            self.cursor_position.x = self.cursor_position.x.saturating_sub(1);
                        }

                        self.movement_data.last_nav_position.x = self.cursor_position.x;
                    }
                },
                Key::Char('k') => {
                    if self.cursor_position.y == 0 {
                        return Ok(());
                    }
                    if let Some(prev_row) = self
                        .document
                        .rows
                        .get(self.cursor_position.y.saturating_sub(1) as usize)
                    {
                        if prev_row.len <= self.movement_data.last_nav_position.x as usize {
                            self.cursor_position.x = prev_row.len.saturating_sub(1) as u16;
                        } else {
                            self.cursor_position.x = self.movement_data.last_nav_position.x;
                        }
                        self.cursor_position.y = self.cursor_position.y.saturating_sub(1);
                    }
                },
                Key::Char('G') => {
                    if let Some(curr_row) = self
                        .document
                        .rows
                        .get(self.document.rows.len().saturating_sub(1) as usize)
                    {
                        self.cursor_position.y = self.document.rows.len().saturating_sub(1) as u16;
                        if curr_row.len <= self.movement_data.last_nav_position.x as usize {
                            self.cursor_position.x = curr_row.len.saturating_sub(1) as u16;
                        } else {
                            self.cursor_position.x = self.movement_data.last_nav_position.x;
                        }
                    }
                },
                Key::Char('0') => {
                    self.cursor_position.x = 0;
                    self.movement_data.last_nav_position.x = self.cursor_position.x;
                },
                Key::Char('H') => {
                    if let Some(curr_row) = self.document.rows.get(self.offset.y as usize)
                    {
                        if curr_row.len < (self.cursor_position.x) as usize {
                            self.cursor_position.x = curr_row.len.saturating_sub(1) as u16;
                        }
                        self.cursor_position.y = self.offset.y;

                        self.movement_data.last_nav_position.x = self.cursor_position.x;
                    }
                },
                Key::Char('M') => {
                    let height = self.net_height;
                    let middle_position = (self.offset.y.saturating_add(height.saturating_div(2)));

                    if let Some(curr_row) = self.document.rows.get(middle_position as usize)
                    {
                        if curr_row.len < (self.cursor_position.x) as usize {
                            self.cursor_position.x = curr_row.len.saturating_sub(1) as u16;
                        }
                        self.cursor_position.y = middle_position as u16;

                        self.movement_data.last_nav_position.x = self.cursor_position.x;
                    }
                },
                Key::Char(':') => {
                    if let Some(command) = self.prompt(|editor, key|  {}, None)? {
                        log!("{}", command);
                    }
                },
                Key::Char('$') => {
                    if let Some(curr_row) = self.document.rows.get(self.cursor_position.y as usize)
                    {
                        self.cursor_position.x = curr_row.len.saturating_sub(1) as u16;
                        self.movement_data.last_nav_position.x = self.cursor_position.x;
                    }
                },
                Key::Char(x) => {
                    if (self.mode == TerminalMode::Normal
                        && !self.document.dirty
                        && !self.document.is_loaded)
                    {
                        if x == 'i' {
                            // switch to insert mode
                        }
                    } else {
                        fsm.run(&x, self);
                    }
                },
                Key::Ctrl('q') => self.should_quit = true,
                Key::Up | Key::Down | Key::Left | Key::Right => {},
                _ => print!("random key pressed!"),
            }
        }
        self.update_selection();
        self.scroll(ScrollDirection::None);

        Ok(())
    }

    pub fn process_insert_mode(&mut self) -> Result<(), io::Error> {
        Ok(())
    }

    pub fn draw_rows(&self) {
        let Size { height, width } = self.terminal.get_size();
        let Position {x: pos_x, y: pos_y} = self.cursor_position;

        print!("{}", termion::color::Bg(termion::color::Black));

        for y in 0..height {
            self.terminal.clear_current_line();
            if let Some(row) = self
                .document
                .rows
                .get(self.offset.y.saturating_add(y) as usize)
            {
                if y == pos_y.saturating_sub(self.offset.y) {
                    print!("{}", termion::color::Bg(termion::color::Rgb(228, 228, 228)));
                    print!("{}" , termion::color::Bg(termion::color::Reset));
                } else {
                    print!("{}" , termion::color::Bg(termion::color::Black));
                }
                self.draw_row(row, width);
                if y == pos_y.saturating_sub(self.offset.y) {
                    let right_pad_len = if width.saturating_add(self.offset.x) > row.len as u16 {
                        (width.saturating_add(self.offset.x) as i32).saturating_sub(row.len as i32).abs() as usize
                    } else {
                        (width.saturating_sub(row.len as u16) as i32).abs() as usize
                    };
                    print!("{}\n\r", " ".repeat(right_pad_len));
                } else {
                    print!("\n\r");
                }

                if y == pos_y.saturating_sub(self.offset.y) {
                    print!("{}", termion::color::Bg(termion::color::Black));
                }
            } else if y == height / 3 {
                self.display_welcome_message();
                return;
            } else {
                if y == 1 {
                    print!("\n\r");
                } else {
                    print!("~\n\r");
                }
                return;
            }
        }

        print!("{}", termion::color::Bg(termion::color::Reset));
    }

    pub fn draw_row(&self, row: &Row, width: u16) {
        if (!row.string.is_empty()) {
            row.render(self.offset.x, self.offset.x.saturating_add(width));
        }
    }

    pub fn scroll(&mut self, intention: ScrollDirection) {
        let Size { height, width } = self.terminal.get_size();
        let Position { x, y } = self.cursor_position;
        let Position {
            x: offset_x,
            y: offset_y,
        } = self.offset;

        if y < offset_y {
            self.offset.y = y;
        } else if y >= offset_y.saturating_add(height) {
            self.offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < offset_x {
            self.offset.x = x;
        } else if x >= offset_x.saturating_add(width) {
            self.offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    pub fn draw_status_bar(&mut self) -> Result<u16, io::Error> {
        let mut rendered_width: usize = 0;
        let Size { width, height } = self.terminal.get_size();
        let Position { y, ..} = self.cursor_position;
        let now = Local::now();
        let circled_dot = format!("{}", " ⊙ ");

        print!("{}", termion::color::Fg(termion::color::White));
        print!("{}", termion::color::Bg(termion::color::AnsiValue(0)));
        print!("{}", circled_dot);
        print!("{}", termion::color::Fg(termion::color::Reset));
        print!("{}", termion::color::Bg(termion::color::Reset));

        rendered_width = rendered_width.saturating_add(circled_dot.graphemes(true).count());

        let mut progress = format!("{}%", ((y.saturating_add(1) as f64 / (self.document.rows.len() as f64)) * 100_f64).ceil());
        progress.push_str("  ");
        progress.truncate(4);

        let mut time_bar = format!(" {:02}:{:02} ", now.hour(), now.minute());
        rendered_width  = rendered_width.saturating_add(time_bar.graphemes(true).count());

        let status_message: String = format!(" {0}/{1} ", progress, self.document.rows.len());
        rendered_width  = rendered_width.saturating_add(status_message.graphemes(true).count());

        let width_diff = width.saturating_sub(rendered_width as u16);
        let space_pad = " ".repeat(width_diff as usize);

        print!("{}", termion::color::Bg(termion::color::LightWhite));
        print!("{}", termion::color::Fg(termion::color::LightBlack));
        print!("{}", space_pad);
        print!("{}", termion::color::Fg(termion::color::Reset));
        print!("{}", termion::color::Bg(termion::color::Reset));

        print!("{}", termion::color::Bg(termion::color::Rgb(124, 120, 127)));
        print!("{}", termion::color::Fg(termion::color::Rgb(244, 240, 247)));
        print!("{}", status_message);
        print!("{}", termion::color::Fg(termion::color::Reset));
        print!("{}", termion::color::Bg(termion::color::Reset));

        print!("{}", termion::color::Fg(termion::color::LightWhite));
        print!("{}", termion::color::Bg(termion::color::Rgb(44, 40, 27)));
        print!("{}", time_bar);
        print!("{}", termion::color::Bg(termion::color::Reset));

        print!("\r");

        Ok((1))
    }

    pub fn get_net_height(&mut self) -> u16 {
        self.net_height
    }

    pub fn draw_message_bar(&mut self, message: Option<&str>) -> Result<u16, io::Error> {
        let mut rendered_width: usize = 0;
        let Size { width, height } = self.terminal.get_size();
        let Position { y, ..} = self.cursor_position;
        let prompt_text = ">|";
        let last_pos = self.cursor_position;
        self.terminal.goto(Position {x: 0, y: height.saturating_add(1)});
        self.terminal.cursor_hide();
        self.terminal.flush()?;

        if message.is_some() {
            print!("{}", termion::color::Fg(termion::color::White));
            print!("{}", termion::color::Bg(termion::color::AnsiValue(0)));
        } else {
            print!("{}", termion::color::Fg(termion::color::LightBlack));
            print!("{}", termion::color::Bg(termion::color::AnsiValue(0)));
        }
        print!("{}", prompt_text);
        print!("{}", termion::color::Fg(termion::color::Reset));
        print!("{}", termion::color::Bg(termion::color::Reset));

        rendered_width = rendered_width.saturating_add(prompt_text.graphemes(true).count()).saturating_add(2);
        self.terminal.goto(Position {x: prompt_text.len() as u16, y: height.saturating_add(1)});

        if let Some(msg) = message {
            const MESSAGE_PAD_LEN: u16 = 2;
            let new_message = msg.chars().rev().take(width.saturating_sub(MESSAGE_PAD_LEN).saturating_sub(rendered_width as u16) as usize).collect::<Vec<_>>().iter().rev().collect::<String>();
            let prompt_message: String = format!(" {}_", new_message);
            print!("{}", prompt_message);
            rendered_width = rendered_width.saturating_add(msg.graphemes(true).count());
        }

        let width_diff = width.saturating_sub(rendered_width as u16);
        let space_pad = " ".repeat(width_diff as usize);

        print!("{}", termion::color::Bg(termion::color::Black));
        print!("{}", termion::color::Fg(termion::color::LightBlack));
        print!("{}", space_pad);
        print!("{}", termion::color::Fg(termion::color::Reset));
        print!("{}", termion::color::Bg(termion::color::Reset));

        print!("\r");
        self.terminal.goto(last_pos);
        self.terminal.cursor_show();
        self.terminal.flush()?;


        Ok((1))
    }

    fn update_selection(&mut self) -> Result<(), io::Error> {
        Ok(())
    }

}

impl Promptable for Editor {
    fn on_prompt_loop_start(&mut self, result: &str) -> Result<(), std::io::Error> {
        self.draw_message_bar(Some(&result));
        self.terminal.flush()?;

        Ok(())
    }
}
