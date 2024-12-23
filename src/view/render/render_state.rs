use syntect::{
    highlighting::{HighlightState, Highlighter},
    parsing::{ParseState, ScopeStack, SyntaxReference},
};

/// 记录某一个渲染状态
#[derive(Debug, Clone, PartialEq)]
pub struct RenderState {
    pub highlight: HighlightState,
    pub parse: ParseState,
}

impl RenderState {
    pub fn new(highlighter: &Highlighter, syntax: &SyntaxReference) -> RenderState {
        Self {
            highlight: HighlightState::new(highlighter, ScopeStack::new()),
            parse: ParseState::new(syntax),
        }
    }
}
