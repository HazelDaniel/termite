use crate::utils::{Position, Size};
use std::fmt;
use std::io::Write;
use termion::raw::{IntoRawMode, RawTerminal};

pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<std::io::Stdout>,
}

impl From<(u16, u16)> for Size {
    fn from(tuple: (u16, u16)) -> Self {
        let (x, y) = tuple;
        Size {
            width: x,
            height: y,
        }
    }
}

impl Default for Terminal {
    fn default() -> Self {
        let stdout = std::io::stdout().into_raw_mode().unwrap();
        let Size { width, height } = Size::from(termion::terminal_size().unwrap());
        Self {
            _stdout: stdout,
            size: Size {
                width,
                height: height.saturating_sub(1),
            },
        }
    }
}

impl Terminal {
    pub fn get_std_buffer(&self) -> &RawTerminal<std::io::Stdout> {
        &self._stdout
    }

    pub fn get_size(&self) -> Size {
        self.size
    }

    pub fn goto(&self, dest: Position) {
        let Position { x, y } = dest;
        print!(
            "{}",
            termion::cursor::Goto(x.saturating_add(1), y.saturating_add(1))
        );
    }

    pub fn clear_screen(&self) {
        print!("{}", termion::clear::All);
    }

    pub fn clear_current_line(&self) {
        print!("{}", termion::clear::CurrentLine);
    }

    pub fn cursor_hide(&self) {
        print!("{}", termion::cursor::Hide);
    }

    pub fn cursor_show(&self) {
        print!("{}", termion::cursor::Show);
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self._stdout.flush()?;
        Ok(())
    }
}
