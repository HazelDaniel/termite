use std::io;
use std::io::{Stdout, stdout, stdin, Write, Read};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use crate::utils::{Position, StatusMessage, TerminalMode, Size, die};
use crate::terminal::Terminal;
use crate::config::{DEFAULT_QUIT_TIMES, PACKAGE_VERSION, EDITOR_NAME};
use crate::document::Document;
use crate::row::Row;

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
        }
    }
}

impl Editor {
    pub fn run (&mut self) -> Result<(), io::Error> {
        let mut terminal = self.terminal.get_std_buffer();
        let Size {width, height} = self.terminal.get_size();

        loop {
            match self.refresh_screen() {
                Ok(res) => {},
                Err(error) => die(error),
            };

            if (self.should_quit) {
                // print goodbye message and cleanup
                println!("goodbye!");
                return Ok(())
            }

            match self.process_keys() {
                Ok(res) => {},
                Err(error) => die(error),
            }
        }



        Ok(())
    }

    pub fn refresh_screen (&mut self) -> Result<(), io::Error> {
        let Position {x: offset_x, y: offset_y} = self.offset;
        let Size {width, height} = self.terminal.get_size();

        self.terminal.cursor_hide();
        self.terminal.clear_screen();
        if self.should_quit {
            self.terminal.cursor_show();
            return Ok(())
        }

        self.move_cursor(
            Position {
                x: self.cursor_position.x.saturating_sub(offset_x),
                y: self.cursor_position.y.saturating_sub(offset_y)
            }
        );

        self.draw_rows();
        self.terminal.cursor_show();
        self.terminal.flush()?;

        Ok(())
    }

    pub fn display_welcome_message (&self) {
        let Size {width, height} = self.terminal.get_size();
        let welcome_message: String = "".to_owned() + EDITOR_NAME + ". v" + PACKAGE_VERSION;
        let message_len = welcome_message.len();
        let width_diff = width - message_len as u16;
        let pad_len = width_diff / 2;
        let mut l_pad = " ".repeat(pad_len as usize);
        l_pad.truncate(pad_len.saturating_sub(1) as usize);
        let r_pad = " ".repeat(pad_len as usize);
        print!("~{}{}{}\n\r", l_pad, welcome_message, r_pad);
    }

    pub fn move_cursor (&mut self, pos: Position) -> Result<(), io::Error> {
        let Position {x, y} = pos;

        self.terminal.goto(Position{x, y});

        Ok(())
    }

    pub fn process_keys (&mut self) -> Result<(), io::Error> {

        if self.mode == TerminalMode::Normal {
            self.process_normal_mode()?;
        }

        Ok(())
    }

    pub fn process_normal_mode (&mut self) -> Result<(), io::Error> {

        if let Some(key) = stdin().keys().next() {
            match key? {
                Key::Char('\n') => {
                    print!("{}", "\n\r");
                },
                Key::Char('l') => {
                    self.cursor_position.x = self.cursor_position.x.saturating_add(1);
                },
                Key::Char('j') => {
                    self.cursor_position.y = self.cursor_position.y.saturating_add(1);
                },
                Key::Char('h') => {
                    self.cursor_position.x = self.cursor_position.x.saturating_sub(1);
                },
                Key::Char('k') => {
                    self.cursor_position.y = self.cursor_position.y.saturating_sub(1);
                },
                Key::Char(x) => {
                    if (self.mode == TerminalMode::Normal && !self.document.dirty && !self.document.is_loaded) {
                        if x == 'i' {
                            // switch to insert mode
                        }
                        ()
                    } else {
                        print!("{}", x);
                    }
                },
                Key::Ctrl('q') => self.should_quit = true,
                Key::Up | Key::Down | Key::Left | Key::Right => {},
                _ => print!("random key pressed!")
            }
        }

        Ok(())
    }

    pub fn process_insert_mode (&mut self) -> Result<(), io::Error> {
        Ok(())
    }


    pub fn draw_rows(&mut self) {
        let Size {height, ..} = self.terminal.get_size();
        for y in 0 ..height {
            if let Some(row) = self.document.rows.get(y as usize) {
            } else if y == height / 3 {
                self.display_welcome_message();
            } else {
                if y == 1 {
                    print!("\n\r");
                } else {
                    print!("~\n\r");
                }
            }
        };
        if !self.document.is_loaded {
            self.move_cursor(Position::default());
        }
    }

    pub fn draw_row(&mut self, row: &Row) {
        if (row.string.is_empty()) {
            print!("{}", "~\r");
            return
        }
    }
}