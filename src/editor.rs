use std::io;
use std::io::{Stdout, stdout, stdin, Write, Read};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use crate::utils::{Position, StatusMessage, TerminalMode, Size, die};
use crate::terminal::Terminal;
use crate::config::{DEFAULT_QUIT_TIMES, PACKAGE_VERSION, EDITOR_NAME};
use crate::document::Document;

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

        self.terminal.goto(
            Position {
                x: self.cursor_position.x.saturating_sub(offset_x),
                y: self.cursor_position.y.saturating_sub(offset_y)
            }
        );

        self.terminal.cursor_show();

        Ok(())
    }

    pub fn display_welcome_message () {
        let welcome_message: String = "".to_owned() + EDITOR_NAME + ". v" + PACKAGE_VERSION;
        let message_len = welcome_message.len();

    }

    pub fn goto (&mut self, pos: Position) -> Result<(), io::Error> {
        let Position {x, y} = pos;

        self.cursor_position = Position{x, y};
        self.terminal.goto(Position{x, y});

        Ok(())
    }

    pub fn process_keys (&mut self) -> Result<(), io::Error> {
        if let Some(key) = stdin().keys().next() {
            match key? {
                Key::Char('\n') => {
                    print!("{}", "\n\r");
                },
                Key::Char(x) => {print!("{}", x)},
                Key::Ctrl('q') => self.should_quit = true,
                Key::Up | Key::Down | Key::Left | Key::Right => {},
                _ => print!("random key pressed!")
            }
        }

        self.terminal.flush()?;

        Ok(())
    }
}