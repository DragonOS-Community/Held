use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, sync::MutexGuard};

use crate::utils::ui::uicore::{UiCore, CONTENT_WINSIZE};

lazy_static! {
    // value is regex
    pub static ref PAIRING: HashMap<char, Regex> = {
        let mut m = HashMap::new();
        m.insert('(', Regex::new(r"(?s)\(.*?").unwrap());
        m.insert('[', Regex::new(r"(?s)\[.*?").unwrap());
        m.insert('{', Regex::new(r"(?s)\{.*?").unwrap());
        m.insert('<', Regex::new(r"(?s)<.*?").unwrap());
        m.insert('\'', Regex::new(r"(?s)\'(.*?)\'").unwrap());
        m.insert('"', Regex::new(r#"(?s)"(.*?)""#).unwrap());
        m.insert('`', Regex::new(r"(?s)`(.*?)`").unwrap());
        m.insert(')', Regex::new(r"\)").unwrap());
        m.insert(']', Regex::new(r"\]").unwrap());
        m.insert('}', Regex::new(r"\}").unwrap());
        m.insert('>', Regex::new(r"\>").unwrap());
        m
    };
}

fn is_left(pat: char) -> bool {
    let left = ['(', '[', '{', '<'];
    left.contains(&pat)
}

fn is_right(pat: char) -> bool {
    let right = [')', ']', '}', '>'];
    right.contains(&pat)
}

fn is_quote(pat: char) -> bool {
    let quote = ['\'', '"', '`'];
    quote.contains(&pat)
}

fn get_pair(pat: char) -> char {
    match pat {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        '<' => '>',
        '\'' => '\'',
        '"' => '"',
        '`' => '`',
        ')' => '(',
        ']' => '[',
        '}' => '{',
        '>' => '<',
        _ => unreachable!(),
    }
}
/// 获取括号文本
pub fn find_pair(ui: &mut MutexGuard<UiCore>, pat: u8) -> Option<String> {
    let win_rows = CONTENT_WINSIZE.read().unwrap().rows;
    // 搜索范围为整个屏幕
    let content = ui.buffer.get_range_str((0, 0), (0, win_rows - 1));
    let x = ui.cursor.x();
    let y = ui.cursor.y();
    let offset = ui.buffer.get_offset_by_pos(x, y);
    get_nested_pair(&content, pat as char, offset, ui)
}

/// 获取匹配的括号
/// @param text: 文本
/// @param pat: 括号
/// @param pos: 光标位置转换后的偏移量
/// @return: 匹配的括号文本
fn get_nested_pair(
    text: &str,
    pat: char,
    pos: usize,
    ui: &mut MutexGuard<UiCore>,
) -> Option<String> {
    let regex = PAIRING.get(&pat)?;
    let mtch = regex.find_at(text, pos);

    if let Some(m) = mtch {
        let (start, end) = (m.start(), m.end());
        let new_cursor_start = ui.buffer.get_pos_by_offset(start);

        match pat {
            _ if is_quote(pat) => {
                ui.cursor
                    .move_to(new_cursor_start.0, new_cursor_start.1)
                    .ok()?;
                return Some(m.as_str().to_string());
            }
            _ if is_left(pat) => {
                ui.cursor
                    .move_to(new_cursor_start.0, new_cursor_start.1)
                    .ok()?;
                return find_matching_right(text, pat, start);
            }
            _ if is_right(pat) => {
                return find_matching_left(text, pat, end, ui);
            }
            _ => None,
        }
    } else {
        None
    }
}

fn find_matching_right(text: &str, left_pat: char, start: usize) -> Option<String> {
    let right_pat = get_pair(left_pat);
    let mut stack = Vec::new();
    let end;

    for (idx, c) in text[start..].chars().enumerate() {
        if c == left_pat {
            stack.push(c);
        } else if c == right_pat {
            stack.pop();
            if stack.is_empty() {
                end = idx + start;
                return Some(text[start..=end].to_string());
            }
        }
    }
    None
}

fn find_matching_left(
    text: &str,
    right_pat: char,
    end: usize,
    ui: &mut MutexGuard<UiCore>,
) -> Option<String> {
    let left_pat = get_pair(right_pat);
    let mut stack = Vec::new();
    let chars: Vec<char> = text[..=end].chars().collect();

    for (idx, c) in chars.iter().enumerate().rev() {
        if *c == right_pat {
            stack.push(*c);
        } else if *c == left_pat {
            stack.pop();
            if stack.is_empty() {
                let new_cursor = ui.buffer.get_pos_by_offset(idx);
                ui.cursor.move_to(new_cursor.0, new_cursor.1).ok()?;
                return Some(text[idx..end].to_string());
            }
        }
    }
    None
}
