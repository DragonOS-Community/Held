use std::path::PathBuf;

pub trait Perferences {
    /// 是否自动换行
    fn line_wrapping(&self) -> bool;

    // tab宽度
    fn tab_width(&self) -> usize;

    // 是否使用空格模拟tab
    fn soft_tab(&self) -> bool;

    // 设置的主题文件路径
    fn theme_path(&self) -> PathBuf;

    // 设置的主题名字
    fn theme_name(&self) -> String;
}

#[cfg(test)]
pub struct DummyPerferences;
#[cfg(test)]
impl Perferences for DummyPerferences {
    fn line_wrapping(&self) -> bool {
        true
    }

    fn tab_width(&self) -> usize {
        2
    }

    fn soft_tab(&self) -> bool {
        todo!()
    }

    fn theme_path(&self) -> PathBuf {
        todo!()
    }

    fn theme_name(&self) -> String {
        todo!()
    }
}
