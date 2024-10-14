use std::{borrow::Borrow, fmt};

use serde_yaml::Index;
use unicode_segmentation::UnicodeSegmentation;

use crate::util::{position::Position, range::Range};

/// GapBuffer 增加减少重分配的buffer
/// 在一整块buffer中有一段gap(空闲空间)将整块buffer分为两段，以实现插入删除等操作的高效率（减少重分配）
/// 数据结构：
///     |       data        |               gap         |       data        |
///     |               gap_start------>gap_length<-----|                   |
pub struct GapBuffer {
    data: Vec<u8>,
    gap_start: usize,
    gap_length: usize,
}

impl GapBuffer {
    pub fn new<T: AsRef<str>>(data: T) -> GapBuffer {
        let mut buf = data.as_ref().as_bytes().to_owned();
        let capacity = buf.capacity();
        let gap_start = buf.len();
        let gap_length = capacity - gap_start;
        unsafe {
            buf.set_len(capacity);
        }

        GapBuffer {
            data: buf,
            gap_start: gap_start,
            gap_length: gap_length,
        }
    }

    pub fn insert(&mut self, data: &str, position: &Position) {
        if data.len() > self.gap_length {
            // 先将gap区域移动到最后，不然会出现两段gap
            let offset = self.data.capacity();
            self.move_gap(offset);
            // 扩容
            self.data.reserve(data.len());

            let capacity = self.data.capacity();
            self.gap_length = capacity - self.gap_start;
            unsafe {
                self.data.set_len(capacity);
            }
        }

        let offset = match self.find_offset(position) {
            Some(offset) => offset,
            None => return,
        };
        // println!("{:?}", self.data);
        self.move_gap(offset);
        // println!("{:?}", self.data);
        self.write_to_gap(data);
        // println!("{:?}", self.data);
        // println!("start {} length {}", self.gap_start, self.gap_length);
    }

    pub fn read(&self, range: &Range) -> Option<String> {
        let start_offset = match self.find_offset(&range.start()) {
            Some(offset) => offset,
            None => return None,
        };

        let end_offset = match self.find_offset(&range.end()) {
            Some(offset) => offset,
            None => return None,
        };

        let data = if start_offset < self.gap_start && end_offset > self.gap_start {
            let mut data =
                String::from_utf8_lossy(&self.data[start_offset..self.gap_start]).into_owned();
            data.push_str(
                String::from_utf8_lossy(&self.data[self.gap_start + self.gap_length..end_offset])
                    .borrow(),
            );
            data
        } else {
            String::from_utf8_lossy(&self.data[start_offset..end_offset]).into_owned()
        };

        Some(data)
    }

    // | data | gap |   data    |
    pub fn delete(&mut self, range: &Range) {
        let start_offset = match self.find_offset(&range.start()) {
            Some(offset) => offset,
            None => return,
        };

        self.move_gap(start_offset);

        match self.find_offset(&range.end()) {
            Some(offset) => {
                self.gap_length = offset - self.gap_start;
            }
            None => {
                // 确定delete后gap长度

                // 尝试跳过一行查找，若找到，则end在原gap区域中，若找不到，则end超出data范围或者在末尾
                let start_of_next_line = Position {
                    line: range.end().line + 1,
                    offset: 0,
                };

                match self.find_offset(&start_of_next_line) {
                    Some(offset) => {
                        self.gap_length = offset - self.gap_start;
                    }
                    None => {
                        self.gap_length = self.data.len() - self.gap_start;
                    }
                }
            }
        }
    }

    pub fn in_bounds(&self, position: &Position) -> bool {
        return self.find_offset(position).is_some();
    }

    /// 将对应的position映射为具体在buffer中的offset
    pub fn find_offset(&self, position: &Position) -> Option<usize> {
        let first = String::from_utf8_lossy(&self.data[..self.gap_start]);
        let mut line = 0;
        let mut line_offset = 0;
        for (offset, grapheme) in (*first).grapheme_indices(true) {
            if line == position.line && line_offset == position.offset {
                return Some(offset);
            }

            if grapheme == "\n" {
                line += 1;
                line_offset = 0;
            } else {
                line_offset += 1;
            }
        }

        // |    data1    |   gap  |   data2    |
        // 当data1最后个字符为'\n'时，若刚好匹配到，则实际offset为data2的开头
        if line == position.line && line_offset == position.offset {
            return Some(self.gap_start + self.gap_length);
        }

        let second = String::from_utf8_lossy(&self.data[self.gap_start + self.gap_length..]);
        for (offset, grapheme) in (*second).grapheme_indices(true) {
            if line == position.line && line_offset == position.offset {
                return Some(self.gap_start + self.gap_length + offset);
            }

            if grapheme == "\n" {
                line += 1;
                line_offset = 0;
            } else {
                line_offset += 1;
            }
        }

        // |    data1    |   gap  |   data2    |
        // 当data2最后个字符为'\n'时，若刚好匹配到，则实际offset为data2的结尾
        if line == position.line && line_offset == position.offset {
            return Some(self.data.len());
        }

        None
    }

    pub fn move_gap(&mut self, offset: usize) {
        if self.gap_length == 0 {
            self.gap_start = offset;
            return;
        }

        match offset.cmp(&self.gap_start) {
            std::cmp::Ordering::Less => {
                for index in (offset..self.gap_start).rev() {
                    self.data[index + self.gap_length] = self.data[index];
                }

                self.gap_start = offset;
            }
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => {
                for index in self.gap_start + self.gap_length..offset {
                    self.data[index - self.gap_length] = self.data[index];
                }

                self.gap_start = offset - self.gap_length;
            }
        }
    }

    // 写gap区域
    fn write_to_gap(&mut self, data: &str) {
        assert!(self.gap_length >= data.bytes().len());
        for byte in data.bytes() {
            self.data[self.gap_start] = byte;
            self.gap_start += 1;
            self.gap_length -= 1;
        }
    }
}

impl fmt::Display for GapBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let first_half = String::from_utf8_lossy(&self.data[..self.gap_start]);
        let second_half = String::from_utf8_lossy(&self.data[self.gap_start + self.gap_length..]);

        write!(f, "{}{}", first_half, second_half)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        buffer::{GapBuffer, Position},
        util::range::Range,
    };

    #[test]
    fn move_gap_works() {
        let mut gb = GapBuffer::new("This is a test.");
        gb.move_gap(0);
        assert_eq!(gb.to_string(), "This is a test.");
    }

    #[test]
    fn inserting_at_the_start_works() {
        let mut gb = GapBuffer::new("toolkit");

        // This insert serves to move the gap to the start of the buffer.
        gb.insert(" ", &Position { line: 0, offset: 0 });
        assert_eq!(gb.to_string(), " toolkit");

        // We insert enough data (more than twice original capacity) to force
        // a re-allocation, which, with the gap at the start and when handled
        // incorrectly, will create a split/two-segment gap. Bad news.
        gb.insert("scribe text", &Position { line: 0, offset: 0 });
        assert_eq!(gb.to_string(), "scribe text toolkit");
    }

    #[test]
    fn inserting_in_the_middle_works() {
        let mut gb = GapBuffer::new("    editor");

        // Same deal as above "at the start" test, where we want to move
        // the gap into the middle and then force a reallocation to check
        // that pre-allocation gap shifting is working correctly.
        gb.insert(" ", &Position { line: 0, offset: 4 });
        gb.insert("scribe", &Position { line: 0, offset: 4 });
        assert_eq!(gb.to_string(), "    scribe editor");
    }

    #[test]
    fn inserting_at_the_end_works() {
        let mut gb = GapBuffer::new("This is a test.");
        gb.insert(
            " Seriously.",
            &Position {
                line: 0,
                offset: 15,
            },
        );
        assert_eq!(gb.to_string(), "This is a test. Seriously.");
    }

    #[test]
    fn inserting_in_different_spots_twice_works() {
        let mut gb = GapBuffer::new("This is a test.");
        gb.insert("Hi. ", &Position { line: 0, offset: 0 });
        gb.insert(
            " Thank you.",
            &Position {
                line: 0,
                offset: 19,
            },
        );
        assert_eq!(gb.to_string(), "Hi. This is a test. Thank you.");
    }

    #[test]
    fn inserting_at_an_invalid_position_does_nothing() {
        let mut gb = GapBuffer::new("This is a test.");
        gb.insert(
            " Seriously.",
            &Position {
                line: 0,
                offset: 35,
            },
        );
        assert_eq!(gb.to_string(), "This is a test.");
    }

    #[test]
    fn inserting_after_a_grapheme_cluster_works() {
        let mut gb = GapBuffer::new("scribe नी");
        gb.insert(" library", &Position { line: 0, offset: 8 });
        assert_eq!(gb.to_string(), "scribe नी library");
    }

    #[test]
    fn deleting_works() {
        let mut gb = GapBuffer::new("This is a test.\nSee what happens.");
        let start = Position { line: 0, offset: 8 };
        let end = Position { line: 1, offset: 4 };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "This is what happens.");
    }

    #[test]
    fn inserting_then_deleting_at_the_start_works() {
        let mut gb = GapBuffer::new("");
        gb.insert("This is a test.", &Position { line: 0, offset: 0 });
        let start = Position { line: 0, offset: 0 };
        let end = Position { line: 0, offset: 1 };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "his is a test.");
    }

    #[test]
    fn deleting_immediately_after_the_end_of_the_gap_widens_the_gap() {
        let mut gb = GapBuffer::new("This is a test.");
        let mut start = Position { line: 0, offset: 8 };
        let mut end = Position { line: 0, offset: 9 };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "This is  test.");
        assert_eq!(gb.gap_length, 1);

        start = Position { line: 0, offset: 9 };
        end = Position {
            line: 0,
            offset: 10,
        };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "This is  est.");
        assert_eq!(gb.gap_length, 2);
    }

    #[test]
    fn deleting_to_an_out_of_range_line_deletes_to_the_end_of_the_buffer() {
        let mut gb = GapBuffer::new("scribe\nlibrary");
        let start = Position { line: 0, offset: 6 };
        let end = Position {
            line: 2,
            offset: 10,
        };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "scribe");
    }

    #[test]
    fn deleting_to_an_out_of_range_column_deletes_to_the_end_of_the_buffer() {
        let mut gb = GapBuffer::new("scribe\nlibrary");
        let start = Position { line: 0, offset: 0 };
        let end = Position {
            line: 0,
            offset: 100,
        };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "library");
    }

    #[test]
    fn deleting_after_a_grapheme_cluster_works() {
        let mut gb = GapBuffer::new("scribe नी library");
        let start = Position { line: 0, offset: 8 };
        let end = Position {
            line: 0,
            offset: 16,
        };
        gb.delete(&Range::new(start, end));
        assert_eq!(gb.to_string(), "scribe नी");
    }

    #[test]
    fn read_does_not_include_gap_contents_when_gap_is_at_start_of_range() {
        // Create a buffer and a range that captures the first character.
        let mut gb = GapBuffer::new("scribe");
        let range = Range::new(
            Position { line: 0, offset: 0 },
            Position { line: 0, offset: 1 },
        );

        // Delete the first character, which will move the gap buffer to the start.
        gb.delete(&range);
        assert_eq!(gb.to_string(), "cribe");

        // Ask for the first character, which would include the deleted
        // value, if the read function isn't smart enough to skip it.
        assert_eq!(gb.read(&range).unwrap(), "c");
    }

    #[test]
    fn read_does_not_include_gap_contents_when_gap_is_in_middle_of_range() {
        let mut gb = GapBuffer::new("scribe");

        // Delete data from the middle of the buffer, which will move the gap there.
        gb.delete(&Range::new(
            Position { line: 0, offset: 2 },
            Position { line: 0, offset: 4 },
        ));
        assert_eq!(gb.to_string(), "scbe");

        // Request a range that extends from the start to the finish.
        let range = Range::new(
            Position { line: 0, offset: 0 },
            Position { line: 0, offset: 4 },
        );
        assert_eq!(gb.read(&range).unwrap(), "scbe");
    }

    #[test]
    fn reading_after_a_grapheme_cluster_works() {
        let gb = GapBuffer::new("scribe नी library");
        let range = Range::new(
            Position { line: 0, offset: 8 },
            Position {
                line: 0,
                offset: 16,
            },
        );
        assert_eq!(gb.read(&range).unwrap(), " library");
    }

    #[test]
    fn in_bounds_considers_grapheme_clusters() {
        let gb = GapBuffer::new("scribe नी library");
        let in_bounds = Position {
            line: 0,
            offset: 16,
        };
        let out_of_bounds = Position {
            line: 0,
            offset: 17,
        };
        assert!(gb.in_bounds(&in_bounds));
        assert!(!gb.in_bounds(&out_of_bounds));
    }
}
