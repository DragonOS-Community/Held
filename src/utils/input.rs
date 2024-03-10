use std::io::{self, Read};

pub struct Input;

impl Input {
    pub fn wait_keydown() -> io::Result<KeyEventType> {
        let buf: &mut [u8] = &mut [0; 8];
        let count = io::stdin().read(buf)?;
        Ok(KeyCodeParser::parse(&buf[0..count]))
    }
}

struct KeyCodeParser;

impl KeyCodeParser {
    pub fn parse(bytes: &[u8]) -> KeyEventType {
        if bytes[0] == 224 {
            // 控制字符
            return Self::parse_ctrl(&bytes[1..]);
        }
        match bytes {
            // Enter key
            b"\n" => KeyEventType::Enter,
            // Tab key
            b"\t" => KeyEventType::Tab,
            // Esc
            [0] => KeyEventType::Esc,

            [8] => KeyEventType::Backspace,

            // ASCII 字符
            [byte] if *byte >= 32 && *byte <= 126 => KeyEventType::Common(bytes[0]),
            // Unknown bytes
            bytes => {
                error!("unknown bytes {bytes:?}");
                KeyEventType::Unknown(bytes.to_vec())
            }
        }
    }

    fn parse_ctrl(bytes: &[u8]) -> KeyEventType {
        match bytes {
            [72] => KeyEventType::Up,
            [80] => KeyEventType::Down,
            [75] => KeyEventType::Left,
            [77] => KeyEventType::Right,
            bytes => {
                error!("unknown ctrl bytes {bytes:?}");
                KeyEventType::Unknown(bytes.to_vec())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum KeyEventType {
    Common(u8),

    Up,
    Down,
    Right,
    Left,

    Enter,
    Tab,
    Backspace,

    Esc,

    Unknown(Vec<u8>),
}
