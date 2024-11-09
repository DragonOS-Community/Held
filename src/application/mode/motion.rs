use held_core::utils::position::Position;

pub fn locate_next_words_begin(count: usize, str: &str, current_pos: &Position) -> Position {
    let s = str.as_bytes();
    let mut left = 0;
    let mut right = left;
    for _ in 0..count {
        while left <= right && right < s.len() {
            let lchar = s[left] as char;
            let rchar = s[right] as char;
            if lchar.is_alphanumeric() {
                left += 1;
                right += 1;
                continue;
            }
            if rchar.is_alphanumeric() {
                left = right;
                break;
            }
            right += 1;
        }
    }
    right = right.min(s.len() - 1);
    // 向下移动的行数
    let down_line = str[..right].matches('\n').count();

    let new_offset = if let Some(idx) = str[..right].rfind('\n') {
        // 即目标位置到行首的距离
        right - idx - 1
    } else {
        // 新旧行之间没有换行符
        current_pos.offset + right
    };
    let pos = Position::new(current_pos.line + down_line, new_offset);
    return pos;
}

pub fn locate_previous_words(count: usize, str: &str, current_pos: &Position) -> Position {
    let s = str.as_bytes();
    let mut left = s.len() - 1;
    let mut right = left;
    for _ in 0..count {
        while left <= right && left > 0 {
            let lchar = s[left] as char;
            let rchar = s[right] as char;
            if !rchar.is_alphanumeric() {
                left -= 1;
                right -= 1;
                continue;
            }
            if !lchar.is_alphanumeric() {
                right = left;
                break;
            }
            left -= 1;
        }
    }
    let up_line = str[left..].matches('\n').count();
    let new_line = current_pos.line - up_line;
    let new_line_len = str.lines().nth(new_line).unwrap().len();
    let new_offset = if let Some(back_offset) = str[left..].find('\n') {
        // back_offset为目标位置到行尾的距离
        // new_line_len - back_offset为目标位置到行首的距离
        new_line_len - back_offset
    } else {
        // 新旧行之间没有换行符
        current_pos.offset - (s.len() - left)
    };
    return Position::new(new_line, new_offset);
}

pub fn locate_next_words_end(count: usize, str: &str, current_pos: &Position) -> Position {
    let s = str.as_bytes();
    let mut left = 0;
    let mut right = left;
    let mut tmp_pos = right;
    for _ in 0..count {
        while left <= right && right < s.len() {
            let lchar = s[left] as char;
            let rchar = s[right] as char;
            if !lchar.is_alphanumeric() {
                left += 1;
                right += 1;
                continue;
            }
            if !rchar.is_alphanumeric() {
                if right == tmp_pos + 1 {
                    left = right;
                    continue;
                }
                right -= 1;
                left = right;
                tmp_pos = right;
                break;
            }
            right += 1;
        }
    }
    right = right.min(s.len() - 1);
    // 向下移动的行数
    let down_line = str[..right].matches('\n').count();

    let new_offset = if let Some(idx) = str[..right].rfind('\n') {
        // 即目标位置到行首的距离
        right - idx - 1
    } else {
        // 新旧行之间没有换行符
        current_pos.offset + right
    };
    let pos = Position::new(current_pos.line + down_line, new_offset);
    return pos;
}
#[cfg(test)]
mod tests {
    #[test]
    fn next_word_test() {
        let mut v = Vec::new();
        v.push("pub fn locate_next_words(count: usize, str\n: &str) -> Position {");
        v.push(stringify!(let new_offset = if let Some(idx) = str[..mt
            ch.start()].rfind('\n')));
        v.push(";\nmod utils;\nmod view;\nmod workspace;\n");

        assert_eq!(next_word_search(1, v[0], 0), 4);
        assert_eq!(next_word_search(1, v[0], 4), 7);

        assert_eq!(next_word_search(1, v[1], 0), 4);
        assert_eq!(next_word_search(1, v[1], 3), 4);

        assert_eq!(next_word_search(1, v[2], 0), 2);
        assert_eq!(next_word_search(2, v[2], 0), 6);
    }

    fn next_word_search(count: usize, str: &str, at: usize) -> usize {
        let s = str.as_bytes();
        let mut left = at;
        let mut right = left;
        for _ in 0..count {
            while left <= right && right < s.len() {
                let lchar = s[left] as char;
                let rchar = s[right] as char;
                if rchar.is_ascii_punctuation() && right != at {
                    break;
                }
                if lchar.is_alphanumeric() {
                    left += 1;
                    right += 1;
                    continue;
                }
                if rchar.is_alphanumeric() {
                    left = right;
                    break;
                }
                right += 1;
            }
        }
        return right;
    }
}
