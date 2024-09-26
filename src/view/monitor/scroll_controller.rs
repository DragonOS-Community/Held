use std::sync::Arc;

use crate::view::terminal::Terminal;

/// 对于滚动操作的抽象对象
///
/// 外部通过line_offset方法获取滚动后buffer的offset
pub struct ScrollController {
    terminal: Arc<dyn Terminal>,
}

impl ScrollController {
    pub fn new(terminal: Arc<dyn Terminal>) -> ScrollController {
        ScrollController { terminal }
    }

    // 将buffer指针指向的行滚动到显示区域顶部
    pub fn scroll_into_monitor(&mut self) {
        todo!()
    }

    // 将buffer指针指向的行滚动到显示区域中间区域
    pub fn scroll_to_center(&mut self) {
        todo!()
    }

    // 向上滚动n行
    pub fn scroll_up(&mut self, line_count: usize) {
        todo!()
    }

    // 向下滚动n行
    pub fn scroll_down(&mut self, line_count: usize) {
        todo!()
    }

    // 返回当前的offset
    pub fn line_offset(&mut self) -> usize {
        todo!()
    }
}
