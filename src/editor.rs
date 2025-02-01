use std::io;
use std::io::{stdin, stdout, Read, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use crate::config::{DEFAULT_QUIT_TIMES, EDITOR_NAME, PACKAGE_VERSION};
use crate::document::Document;
use crate::row::Row;
use crate::terminal::Terminal;
use crate::utils::{
    die, HighlightingOptions, MovementData, Position, Size, StatusMessage, TerminalMode,
};

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: Option<StatusMessage>,
    quit_times: u8,
    highlighted_word: Option<String>,
    mode: TerminalMode,
    movement_data: MovementData,
}

impl Default for Editor {
    fn default() -> Self {
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
        }
    }
}

impl Editor {
    pub fn run(&mut self) -> Result<(), io::Error> {
        let mut terminal = self.terminal.get_std_buffer();
        let Size { width, height } = self.terminal.get_size();
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

            match self.process_keys() {
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
            Some(offset_y.saturating_add(height)),
        );
        self.draw_rows();

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

    pub fn process_keys(&mut self) -> Result<(), io::Error> {
        if self.mode == TerminalMode::Normal {
            self.process_normal_mode()?;
        }

        Ok(())
    }

    pub fn process_normal_mode(&mut self) -> Result<(), io::Error> {
        let Size { height, .. } = self.terminal.get_size();

        if let Some(key) = stdin().keys().next() {
            match key? {
                Key::Char('\n') => {
                    print!("{}", "\n\r");
                }
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
                }
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
                }
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
                }
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
                }
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
                }
                Key::Char('0') => {
                    self.cursor_position.x = 0;
                    self.movement_data.last_nav_position.x = self.cursor_position.x;
                }
                Key::Char('$') => {
                    if let Some(curr_row) = self.document.rows.get(self.cursor_position.y as usize)
                    {
                        self.cursor_position.x = curr_row.len.saturating_sub(1) as u16;
                        self.movement_data.last_nav_position.x = self.cursor_position.x;
                    }
                }
                Key::Char(x) => {
                    if (self.mode == TerminalMode::Normal
                        && !self.document.dirty
                        && !self.document.is_loaded)
                    {
                        if x == 'i' {
                            // switch to insert mode
                        }
                    } else {
                        print!("{}", x);
                    }
                }
                Key::Ctrl('q') => self.should_quit = true,
                Key::Up | Key::Down | Key::Left | Key::Right => {}
                _ => print!("random key pressed!"),
            }
        }
        self.scroll();

        Ok(())
    }

    pub fn process_insert_mode(&mut self) -> Result<(), io::Error> {
        Ok(())
    }

    pub fn draw_rows(&self) {
        let Size { height, width } = self.terminal.get_size();

        for y in 0..height {
            self.terminal.clear_current_line();
            if let Some(row) = self
                .document
                .rows
                .get(self.offset.y.saturating_add(y) as usize)
            {
                self.draw_row(row, width);
            } else if y == height / 3 {
                self.display_welcome_message();
            } else {
                if y == 1 {
                    print!("\n\r");
                } else {
                    print!("~\n\r");
                }
            }
        }
    }

    pub fn draw_row(&self, row: &Row, width: u16) {
        if (!row.string.is_empty()) {
            row.render(self.offset.x, self.offset.x.saturating_add(width));
            print!("{}", "\n\r");
        } else {
            print!("{}", "\n\r");
        }
    }

    pub fn scroll(&mut self) {
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
}
