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
        let mut buffer = ContentRenderBuffer::new(Rectangle {
            position: interface::cursor::screen_cursor_position(),
            width: 1,
            height: 1,
        });

        buffer.set_cell(
            Position::new(0, 0),
            Some(Cell::new('!', Colors::Warning, CharStyle::Bold)),
        );

        vec![buffer]
    }
}
