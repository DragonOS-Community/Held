#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Distance {
    pub lines: usize,
    pub offset: usize,
}

impl Distance {
    /// 计算字符串覆盖的距离
    ///
    /// /// # Examples
    ///
    /// ```
    /// use crate::buffer::distance::Distance;
    ///
    /// let data = "data\ndistance";
    /// assert_eq!(Distance::of_str(data), Distance{
    ///     lines: 1,
    ///     offset: 8
    /// });
    /// ```
    pub fn of_str(from: &str) -> Distance {
        Distance {
            lines: from.chars().filter(|&c| c == '\n').count(),
            offset: from.split('\n').last().map(|l| l.len()).unwrap_or(0),
        }
    }
}
