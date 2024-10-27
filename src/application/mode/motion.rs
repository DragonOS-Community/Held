use held_core::utils::position::Position;

pub fn locate_next_words_begin(
    count: usize,
    str: &str,
    current_pos: &Position,
) -> Option<Position> {
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
    if right == s.len() {
        return None;
    }
    let add_line = str[..right].matches('\n').count();
    let new_offset = if let Some(idx) = str[..right].rfind('\n') {
        right - idx - 1
    } else {
        current_pos.offset + right
    };
    let pos = Position {
        line: current_pos.line + add_line,
        offset: new_offset,
    };
    return Some(pos);
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
