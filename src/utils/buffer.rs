use std::{
    collections::HashMap,
    io,
    ops::Deref,
    sync::{
        atomic::{AtomicUsize, Ordering},
        RwLock,
    },
};

use bitflags::bitflags;
use crossterm::style::Color;

use super::{
    style::StyleManager,
    ui::uicore::{APP_INFO, CONTENT_WINSIZE},
};

#[derive(Debug, Default, Clone)]
pub struct LineBuffer {
    id: usize,

    pub data: Vec<u8>,

    // 是否被标记
    pub flags: LineState,
}

impl Deref for LineBuffer {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl LineBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        static LINE_ID_ALLOCTOR: AtomicUsize = AtomicUsize::new(0);
        Self {
            id: LINE_ID_ALLOCTOR.fetch_add(1, Ordering::SeqCst),
            data,
            flags: LineState::empty(),
        }
    }

    #[inline]
    pub fn remove(&mut self, idx: usize) {
        self.data.remove(idx);
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn extend(&mut self, other: LineBuffer) {
        self.data.extend(other.data)
    }

    #[inline]
    pub fn insert(&mut self, idx: usize, data: u8) {
        self.data.insert(idx, data)
    }
}

#[derive(Debug, Default)]
pub struct EditBuffer {
    buf: RwLock<Vec<LineBuffer>>,

    // 记录当前页第一行对应buf中的index
    offset: AtomicUsize,

    // 记录被标记的行,行ID -> 行index
    flag_lines: RwLock<HashMap<usize, usize>>,

    // 记录锁定行
    locked_lines: RwLock<HashMap<usize, usize>>,
}

impl EditBuffer {
    pub fn new(buf: Vec<u8>) -> Self {
        let mut lines = buf
            .split_inclusive(|x| *x == '\n' as u8)
            .map(|slice| slice.to_vec())
            .collect::<Vec<Vec<_>>>();

        let last = lines.last();
        if last.is_none() {
            lines.push(vec!['\n' as u8])
        } else {
            let last = last.unwrap();
            if !last.ends_with(&['\n' as u8]) {
                lines.last_mut().unwrap().push('\n' as u8)
            }
        }

        let mut buf = Vec::new();
        for v in lines {
            buf.push(LineBuffer::new(v));
        }

        Self {
            buf: RwLock::new(buf),
            offset: AtomicUsize::new(0),
            flag_lines: RwLock::new(HashMap::new()),
            locked_lines: RwLock::new(HashMap::new()),
        }
    }

    #[inline]
    pub fn set_offset(&self, mut offset: usize) {
        offset = offset.min(self.buf.read().unwrap().len() - 1);
        self.offset.store(offset, Ordering::SeqCst);
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset.load(Ordering::SeqCst)
    }

    pub fn line_count(&self) -> usize {
        self.buf.read().unwrap().len()
    }

    /// 获取一部分上下文
    pub fn get_content(
        &self,
        mut start_y: usize,
        mut line_count: usize,
    ) -> Option<Vec<LineBuffer>> {
        start_y += self.offset.load(Ordering::SeqCst);
        line_count = line_count.min(self.line_count() - start_y);
        let buf = self.buf.read().unwrap();
        if start_y > buf.len() {
            return None;
        }
        let end = buf.len().min(start_y + line_count);

        let mut ret = Vec::with_capacity(end - start_y);
        ret.resize(end - start_y, LineBuffer::default());
        ret[..].clone_from_slice(&buf[start_y..end]);
        Some(ret)
    }

    pub fn get_linesize(&self, line: u16) -> u16 {
        let buf = self.buf.read().unwrap();
        let line = buf.get(self.offset.load(Ordering::SeqCst) + line as usize);
        if line.is_none() {
            return 0;
        }

        let line = line.unwrap();

        line.data.len() as u16
    }

    /// 外部接口，本结构体内部方法不应该使用，因为涉及offset计算
    pub fn remove_char(&self, x: u16, y: u16) {
        let mut buf = self.buf.write().unwrap();
        let line = buf.get_mut(self.offset.load(Ordering::SeqCst) + y as usize);
        if line.is_none() {
            return;
        }

        line.unwrap().remove(x as usize);
    }

    pub fn remove_str(&self, x: u16, y: u16, n: usize) {
        let mut buf = self.buf.write().unwrap();
        let line = buf.get_mut(self.offset.load(Ordering::SeqCst) + y as usize);
        if line.is_none() {
            return;
        }
        let x = x as usize;
        line.unwrap().data.drain(x..x + n);
    }

    /// 获取一份对应行的拷贝
    pub fn get_line(&self, line: u16) -> LineBuffer {
        let buf = self.buf.read().unwrap();
        let line = buf.get(self.offset.load(Ordering::SeqCst) + line as usize);
        if line.is_none() {
            LineBuffer::default()
        } else {
            line.unwrap().clone()
        }
    }

    /// 将某行数据与上一行合并
    /// 返回合并是否成功,以及被合并行之前的长度
    pub fn merge_line(&self, line: u16) -> (bool, usize) {
        let line = self.offset.load(Ordering::SeqCst) + line as usize;
        if line == 0 {
            // 没有上一行
            return (false, 0);
        }

        let mut buf = self.buf.write().unwrap();
        let cur_line = buf.get(line as usize).unwrap().clone();

        let previous_line = buf.get_mut(line - 1).unwrap();

        if previous_line.flags.contains(LineState::LOCKED)
            || cur_line.flags.contains(LineState::LOCKED)
        {
            APP_INFO.lock().unwrap().info = "Row is locked".to_string();
            return (false, 0);
        }

        let p_len = previous_line.size();
        // 移除最后的\n
        previous_line.remove(p_len - 1);
        previous_line.extend(cur_line);

        buf.remove(line as usize);

        return (true, p_len - 1);
    }

    /// 屏幕坐标
    #[inline]
    pub fn insert_char(&self, ch: u8, x: u16, y: u16) {
        let mut buf = self.buf.write().unwrap();
        if buf.len() > 0 {
            let line = buf.get_mut(self.offset() + y as usize).unwrap();
            line.insert(x as usize, ch);
        } else {
            buf.push(LineBuffer::new(vec![ch]));
        }
    }

    #[inline]
    pub fn all_buffer(&self) -> Vec<LineBuffer> {
        self.buf.read().unwrap().clone()
    }

    /// 输入enter时buf的更新操作
    pub fn input_enter(&self, x: u16, y: u16) {
        let y = self.offset.load(Ordering::SeqCst) + y as usize;

        let mut buf = self.buf.write().unwrap();
        let linesize = buf.get(y).unwrap().size();
        if x as usize == linesize {
            buf.insert(y, LineBuffer::new(vec!['\n' as u8]));
        }

        let oldline = buf.get_mut(y).unwrap();
        let mut newline = Vec::new();
        newline.extend_from_slice(&oldline.data[x as usize..]);

        oldline.data.resize(x as usize, 0);
        oldline.data.push('\n' as u8);

        buf.insert(y + 1, LineBuffer::new(newline));
    }

    pub fn add_line_flags(&self, line_index: usize, flags: LineState) {
        let mut buf = self.buf.write().unwrap();

        let line = buf.get_mut(line_index);

        if line.is_none() {
            return;
        }

        let line = line.unwrap();

        line.flags.insert(flags);

        let mut flag_map = self.flag_lines.write().unwrap();
        if flags.contains(LineState::FLAGED) {
            flag_map.insert(line.id, line_index);
        }

        let mut locked_map = self.locked_lines.write().unwrap();
        if flags.contains(LineState::LOCKED) {
            locked_map.insert(line.id, line_index);
        }
    }

    pub fn remove_line_flags(&self, line_index: usize, flags: LineState) {
        let mut buf = self.buf.write().unwrap();

        let line = buf.get_mut(line_index);

        if line.is_none() {
            return;
        }

        let line = line.unwrap();

        line.flags.remove(flags);

        let mut flag_map = self.flag_lines.write().unwrap();
        if flags.contains(LineState::FLAGED) {
            flag_map.remove(&line.id);
        }

        let mut locked_map = self.locked_lines.write().unwrap();
        if flags.contains(LineState::LOCKED) {
            locked_map.remove(&line.id);
        }
    }

    #[inline]
    pub fn line_flags(&self, line: u16) -> LineState {
        self.get_line(line).flags
    }

    // 定位到指定行数，返回在正文窗口中的y坐标
    pub fn goto_line(&self, mut line_idx: usize) -> u16 {
        let max_line = self.line_count();

        if line_idx > max_line - 1 {
            line_idx = max_line - 1
        }

        let size = *CONTENT_WINSIZE.read().unwrap();

        // 先将其坐标定位在正文中央
        let win_rows = size.rows as usize;
        let mut y = win_rows / 2;

        if line_idx < y {
            self.set_offset(0);
            return line_idx as u16;
        }

        let mut offset = line_idx - y;

        if offset + win_rows > max_line {
            // 最底下无数据，则调整
            let adapt_offset = max_line - win_rows;

            y += offset - adapt_offset;
            offset = adapt_offset;
        }

        self.set_offset(offset);

        y as u16
    }

    /// 删除行,不会删除锁定行，返回删除成功的行数
    pub fn delete_lines(&self, start: usize, mut end: usize) -> usize {
        let max = self.line_count();
        if start >= max {
            return 0;
        }

        end = end.min(max);
        let mut index = start;
        let mut count = 0;
        let mut buffer = self.buf.write().unwrap();

        for _ in start..=end {
            let line = buffer.get(index).unwrap();
            if line.flags.contains(LineState::LOCKED) {
                index += 1;
            } else {
                buffer.remove(index);
                count += 1;
            }
        }

        count
    }

    pub fn delete_line(&self, y: usize) {
        let mut buffer = self.buf.write().unwrap();
        let line = buffer.get(y).unwrap();
        if line.data.is_empty() {
            return;
        }

        if !line.flags.contains(LineState::LOCKED) {
            buffer.remove(y);
        }
    }

    pub fn delete_until_line_beg(&self, x: usize, y: usize) -> Option<usize> {
        let mut buffer = self.buf.write().unwrap();
        let line = buffer.get_mut(y).unwrap();

        if line.data.len() < 2 {
            return None;
        }
        line.data.drain(0..x);
        return Some(x - 1);
    }

    pub fn delete_until_endl(&self, x: usize, y: usize) -> Option<usize> {
        let mut buffer = self.buf.write().unwrap();
        let line = buffer.get_mut(y).unwrap();
        let len = line.data.len();
        if len < 2 {
            return None;
        }
        line.data.drain(x..len - 1);
        return Some(x);
    }

    /// 返回下一个单词的起始位置
    /// 如果为该行最后一单词，返回该行长度
    pub fn search_nextw_begin(&self, x: u16, y: u16) -> usize {
        let mut left = x as usize;
        let mut right = left;
        let linesize = self.get_linesize(y) as usize;
        let buf = self.buf.read().unwrap();
        let line = buf
            .get(self.offset.load(Ordering::SeqCst) + y as usize)
            .unwrap();

        while left <= right && right < linesize {
            let lchar = line[left] as char;
            let rchar = line[right] as char;
            if !(lchar == ' ' || lchar == '\t') {
                left += 1;
                right += 1;
                continue;
            }
            if rchar != ' ' && rchar != '\t' {
                break;
            }
            right += 1;
        }

        return right;
    }

    /// 搜索下一个单词的末尾
    /// 如果为该行最后一单词，返回该行长度
    pub fn search_nextw_end(&self, x: u16, y: u16) -> usize {
        let mut left = x as usize;
        let mut right = left;
        let linesize = self.get_linesize(y) as usize;
        let buf = self.buf.read().unwrap();
        let line = buf
            .get(self.offset.load(Ordering::SeqCst) + y as usize)
            .unwrap();

        while left <= right && right < linesize {
            let lchar = line[left] as char;
            let rchar = line[right] as char;
            if lchar == ' ' || lchar == '\t' {
                left += 1;
                right += 1;
                continue;
            }
            if rchar == ' ' || rchar == '\t' {
                if right == x as usize + 1 {
                    left = right;
                    continue;
                }
                right -= 1;
                break;
            }
            right += 1;
        }

        return right;
    }

    /// 返回前一单词首字母位置，如果是当前行首单词，返回 None
    pub fn search_prevw_begin(&self, x: u16, y: u16) -> Option<usize> {
        let mut left = x as i32;
        let mut right = left;
        let buf = self.buf.read().unwrap();
        let line = buf
            .get(self.offset.load(Ordering::SeqCst) + y as usize)
            .unwrap();

        while left <= right && left >= 0 {
            let lchar = line[left as usize] as char;
            let rchar = line[right as usize] as char;

            if rchar == ' ' || rchar == '\t' {
                left -= 1;
                right -= 1;
                continue;
            }

            if lchar == ' ' || lchar == '\t' {
                if left + 1 == x.into() {
                    right = left;
                    continue;
                }
                return Some(left as usize + 1);
            }

            left -= 1;
        }
        return None;
    }
}

bitflags! {
    #[derive(Debug, Default, Clone,Copy)]
    pub struct LineState: u32 {
        /// 该行被标记
        const FLAGED = 1 << 1;
        /// 锁定该行不能被更改
        const LOCKED = 1 << 2;
    }
}

impl LineState {
    pub fn set_style(&self) -> io::Result<()> {
        if self.contains(Self::FLAGED) {
            StyleManager::set_background_color(Color::Cyan)?;
        }

        if self.contains(Self::LOCKED) {
            StyleManager::set_background_color(Color::DarkRed)?;
        }

        Ok(())
    }
}
