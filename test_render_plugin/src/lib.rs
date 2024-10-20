use held_core::{
    declare_plugin, interface,
    plugin::Plugin,
    utils::{position::Position, rectangle::Rectangle},
    view::{
        colors::Colors,
        render::{cell::Cell, ContentRenderBuffer},
        style::CharStyle,
    },
};

declare_plugin!(RenderTestPlugin, RenderTestPlugin::new);

struct RenderTestPlugin;

impl RenderTestPlugin {
    fn new() -> RenderTestPlugin {
        RenderTestPlugin
    }
}

impl Plugin for RenderTestPlugin {
    fn name(&self) -> &'static str {
        "render test plugin"
    }

    fn init(&self) {}

    fn deinit(&self) {}

    fn on_render_content(&self) -> Vec<ContentRenderBuffer> {
        let cursor_position = interface::cursor::screen_cursor_position();

        let width = 10;
        let mut buffer = ContentRenderBuffer::new(Rectangle {
            position: Position {
                line: cursor_position.line.saturating_sub(1),
                offset: cursor_position.offset - width / 2,
            },
            width,
            height: 1,
        });

        let buffer_str = format!("{}:{}", cursor_position.offset, cursor_position.line);
        buffer.put_buffer(
            Position {
                line: 0,
                offset: (width - buffer_str.len()) / 2,
            },
            buffer_str,
            CharStyle::Bold,
            Colors::Warning,
        );

        vec![buffer]
    }
}
