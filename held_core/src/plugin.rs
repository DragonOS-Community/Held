use crate::view::render::ContentRenderBuffer;

pub trait Plugin {
    fn name(&self) -> &'static str;

    fn init(&self);

    fn deinit(&self);

    // 渲染文本内容部分时会触发该回调，可以返回想要在content中渲染的buffer
    fn on_render_content(&self) -> Vec<ContentRenderBuffer> {
        vec![]
    }
}
