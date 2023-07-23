use crossterm::event::{self, KeyCode, KeyEvent};

pub async fn get_line(channel: &mut russh::Channel<russh::client::Msg>) -> String {
    let mut out = String::new();
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
                            channel.data(strchr.as_bytes()).await.unwrap();
                        }
                        KeyCode::Enter => {
                            channel.data(&b"\n"[..]).await.unwrap();
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
