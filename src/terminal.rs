use crossterm::{
    cursor, execute,
    style::{self, Stylize},
    terminal,
    terminal::size,
    QueueableCommand, 
};
use std::io::{stdout, Write};

#[derive(Debug)]
pub struct Terminal;

impl Terminal {
    const MIN_WIDTH: u16 = 64;
    const MIN_HEIGHT: u16 = 32;

    fn check_bounds(&self, w: u16, h: u16) -> std::result::Result<(), TerminalError> {
        if w < Self::MIN_WIDTH || h < Self::MIN_HEIGHT {
            return Err(TerminalError::ErrorKind(format!(
                "terminal is too small to display screen: {}x{}",
                w, h
            )));
        }
        Ok(())
    }
}

pub trait TerminalBackend {
    type Error;
    fn clear_screen(&mut self) -> std::result::Result<(), Self::Error>;
    fn draw_screen(&mut self, display: &[[u8; 32]; 64]) -> std::result::Result<(), Self::Error>;
}

#[derive(Clone, Debug)]
pub enum TerminalError {
    ErrorKind(String),
}

impl std::fmt::Display for TerminalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TerminalError::ErrorKind(s) => writeln!(f, "error: {}", s)?,
        }
        Ok(())
    }
}

impl std::error::Error for TerminalError {}

impl From<std::io::Error> for TerminalError {
    fn from(err: std::io::Error) -> TerminalError {
        dbg!(err);
        TerminalError::ErrorKind("received an io error".to_string())
    }
}

impl TerminalBackend for Terminal {
    type Error = TerminalError;
    fn clear_screen(&mut self) -> std::result::Result<(), Self::Error> {
        let (width, height) = size()?;
        self.check_bounds(width, height)?;
        execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    fn draw_screen(&mut self, disp: &[[u8; 32]; 64]) -> std::result::Result<(), Self::Error> {
        let (width, height) = size()?;
        self.check_bounds(width, height)?;
        let mut stdout = stdout();
        for (i, row) in disp.iter().enumerate() {
            for (j, pix) in row.iter().enumerate() {
                stdout.queue(cursor::MoveTo(i as u16, j as u16))?;
                if *pix == 0 {
                    stdout.queue(style::PrintStyledContent("█".black()))?;
                } else if *pix == 1 {
                    stdout.queue(style::PrintStyledContent("█".white()))?;
                } else {
                    return Err(TerminalError::ErrorKind(
                        "display pixel set to value other than 0 or 1".to_string(),
                    ));
                }
            }
        }
        stdout.flush()?;
        Ok(())
    }
}
