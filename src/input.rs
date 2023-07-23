use crossterm::event::{self, KeyCode, KeyEvent};
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::{execute, Result};
use std::io::{stdout, Write};

pub fn get_line() -> String {
    let mut out = String::new();
    let mut stdout = stdout();
    loop {
        match event::read() {
            Ok(event::Event::Key(KeyEvent {
                code,
                modifiers: _,
                state: _,
                kind,
            })) => {
                if kind == event::KeyEventKind::Press {
                    match code {
                        KeyCode::Char(c) => {
                            let strchr = String::from(c);
                            out += &strchr;
                            execute!(
                                stdout,
                                SetForegroundColor(Color::White),
                                SetBackgroundColor(Color::Black)
                            )
                            .unwrap();
                            execute!(stdout, Print(strchr)).unwrap();
                            execute!(stdout, ResetColor).unwrap();
                        }
                        KeyCode::Enter => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error occurred: {:?}", e);
                break;
            }
        }
    }
    out
}
