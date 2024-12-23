use held_core::utils::position::Position;

#[derive(Debug, PartialEq)]
pub enum MappedLexeme<'a> {
    Focused(&'a str),
    Blurred(&'a str),
}

/// 词素映射器
/// 在渲染时会优先按照词素映射器映射的风格进行映射
///
/// 例：按照正则表达式搜索文本时，聚焦正则匹配的部分
pub trait LexemeMapper {
    fn map<'x>(&'x mut self, lexeme: &str, position: Position) -> Vec<MappedLexeme<'x>>;
}
