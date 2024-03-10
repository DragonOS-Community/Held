use std::{
    io,
    sync::{atomic::AtomicU16, Arc, Mutex, MutexGuard, Once, RwLock, Weak},
};

use crossterm::{
    style::Color,
    terminal::{self},
};
use lazy_static::lazy_static;

use crate::{
    config::appconfig::AppSetting,
    utils::{
        buffer::EditBuffer, cursor::CursorCrtl, style::StyleManager, terminal::TermManager,
        ui::InfoLevel,
    },
};

#[cfg(feature = "dragonos")]
use crate::utils::input::Input;

use super::{
    mode::mode::{Command, InputMode, Insert, LastLine, ModeType},
    AppInfo,
};

lazy_static! {
    static ref COMMAND: Arc<Command> = Arc::new(Command);
    static ref INSERT: Arc<Insert> = Arc::new(Insert);
    static ref LASTLINE: Arc<LastLine> = Arc::new(LastLine::new());
    pub static ref APP_INFO: Mutex<AppInfo> = Mutex::new(AppInfo {
        level: InfoLevel::Info,
        info: String::new()
    });
}

pub static TAB_SIZE: AtomicU16 = AtomicU16::new(4);

#[derive(Debug, Clone, Copy)]
pub struct WinSize {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug)]
pub struct UiCore {
    pub buffer: Arc<EditBuffer>,
    pub cursor: CursorCrtl,

    #[allow(dead_code)]
    setting: AppSetting,
    container: Weak<Ui>,

    edited: bool,
    edited_once: Once,
}

impl UiCore {
    pub fn new(buf: Arc<EditBuffer>, cursor: CursorCrtl, setting: AppSetting) -> Self {
        Self {
            buffer: buf,
            cursor,
            container: Weak::new(),
            setting,
            edited: false,
            edited_once: Once::new(),
        }
    }

    pub fn edited(&self) -> bool {
        self.edited
    }

    pub fn set_edited(&mut self) {
        self.edited_once.call_once(|| self.edited = true)
    }

    pub fn update_bottom_state_bar(&mut self) -> io::Result<()> {
        let container = self.container.upgrade().unwrap();
        let mode = container.mode.read().unwrap().mode_type();
        if mode == ModeType::LastLine {
            return Ok(());
        }

        let size = *WINSIZE.read().unwrap();

        let store_x = self.cursor.x();
        let store_y = self.cursor.y();

        self.cursor.set_prefix_mode(false);

        DEF_STYLE.read().unwrap().set_cmd_style()?;
        let cmd_y = size.rows - 1;
        self.cursor.move_to_row(cmd_y)?;
        self.cursor.clear_current_line()?;
        self.cursor
            .write_with_pos(format!("{mode:?}"), 0, cmd_y, false)?;

        let (buf_x, buf_y) = (store_x, store_y + 1 + self.buffer.offset() as u16);
        let index_info = format!("row:{buf_y} col:{buf_x}");
        let len = index_info.len() as u16;
        self.cursor
            .write_with_pos(index_info, size.cols - len, cmd_y, false)?;

        self.cursor.set_prefix_mode(true);
        self.cursor.move_to(store_x, store_y)?;

        let mut info = APP_INFO.lock().unwrap();
        info.level.set_style()?;
        self.cursor
            .write_with_pos(&info.info, size.cols / 3, cmd_y, false)?;

        info.reset();
        self.cursor.move_to(store_x, store_y)?;

        StyleManager::reset_color()?;

        Ok(())
    }

    /// 渲染部分文件内容，从y行开始渲染count行
    /// 返回实际渲染行数
    pub fn render_content(&mut self, mut y: u16, mut count: usize) -> io::Result<usize> {
        y += UI_HEAD_OFFSET;
        let content_winsize = *CONTENT_WINSIZE.read().unwrap();

        // 超出正文范围
        if y + count as u16 > content_winsize.rows {
            count = (content_winsize.rows - y) as usize;
        }

        let def_style = *DEF_STYLE.read().unwrap();

        let content = self.buffer.get_content(y as usize, count);

        if content.is_none() {
            return Ok(0);
        }
        let content = content.unwrap();

        // 保存光标
        let pos = self.cursor.store_tmp_pos();

        let tmp = y;

        let num_len = (tmp + content_winsize.rows).to_string().len();

        self.cursor.set_prefix_mode(false);
        for line in content.iter() {
            let str = String::from_utf8_lossy(&line.data).to_string();
            def_style.set_content_style()?;

            // 移动
            self.cursor
                .move_to(num_len as u16 + 2 + CursorCrtl::PREFIX_COL, y)?;
            self.cursor.clear_current_line()?;
            self.cursor.write(str)?;
            y += 1;
            StyleManager::reset_color()?;
        }

        self.cursor.update_line_prefix(&content, tmp, num_len)?;
        self.cursor.set_prefix_mode(true);

        self.cursor.restore_tmp_pos(pos)?;

        self.cursor.highlight(None)?;

        Ok(content.len())
    }

    // 将正文向上滚动count行
    pub fn scroll_up(&mut self, mut count: u16) -> io::Result<()> {
        let winsize = *CONTENT_WINSIZE.read().unwrap();

        let pos = self.cursor.store_tmp_pos();

        // 计算最多还能滚动多少行
        let offset = self.buffer.offset();

        // 最多出两行
        let linecount = self.buffer.line_count();
        if offset + winsize.rows as usize + count as usize >= linecount {
            count = linecount as u16 - offset as u16 - winsize.rows;
        }
        self.buffer.set_offset(offset + count as usize);
        // 将光标移动到滚动后的位置
        self.cursor.move_to_row(winsize.rows - count)?;

        // 执行滚动
        TermManager::scroll_up(count)?;

        // 清除光标以下的内容
        TermManager::clear_under_cursor()?;

        // 渲染count行数据
        self.render_content(self.cursor.y(), count as usize)?;

        self.cursor.restore_tmp_pos(pos)?;

        self.cursor.highlight(Some(self.cursor.y() - count))?;
        Ok(())
    }

    pub fn scroll_down(&mut self, mut count: u16) -> io::Result<()> {
        let pos = self.cursor.store_tmp_pos();

        // 计算最多还能滚动多少行
        let offset = self.buffer.offset();
        if offset < count as usize {
            count = offset as u16;
        }

        self.buffer.set_offset(offset - count as usize);
        // 将光标移动第count行

        // 执行滚动
        TermManager::scroll_down(count)?;

        self.cursor.move_to_row(count - 1)?;
        // 清除光标以上的内容
        TermManager::clear_up_cursor()?;

        // 渲染count行数据
        self.render_content(0, count as usize)?;

        self.cursor.restore_tmp_pos(pos)?;

        self.cursor.highlight(Some(self.cursor.y() + count))?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Ui {
    pub core: Arc<Mutex<UiCore>>,
    pub mode: RwLock<Arc<dyn InputMode>>,
}

lazy_static! {
    pub static ref WINSIZE: RwLock<WinSize> = {
        let size = terminal::size().unwrap();
        RwLock::new(WinSize {
            cols: size.0,
            rows: size.1,
        })
    };
    pub static ref CONTENT_WINSIZE: RwLock<WinSize> = {
        let size = *WINSIZE.read().unwrap();
        RwLock::new(WinSize {
            cols: size.cols,
            rows: size.rows - UI_CMD_HEIGHT - UI_HEAD_OFFSET,
        })
    };
    pub static ref DEF_STYLE: RwLock<UiStyle> = {
        let style = UiStyle {
            content_fg: Some(Color::White),
            content_bg: None,
            cmd_line_fg: Some(Color::White),
            cmd_line_bg: Some(Color::DarkCyan),
        };

        RwLock::new(style)
    };
}

pub static UI_HEAD_OFFSET: u16 = 0;
pub const UI_CMD_HEIGHT: u16 = 1;

impl Ui {
    pub fn new(buf: Arc<EditBuffer>, setting: AppSetting) -> Arc<Self> {
        let mut cursor = CursorCrtl::new(buf.clone(), setting.line);
        cursor.move_to(0, 0).unwrap();

        let core = Arc::new(Mutex::new(UiCore::new(buf, cursor, setting)));
        let ret = Arc::new(Self {
            mode: RwLock::new(Arc::new(Command)),
            core: core.clone(),
        });

        core.lock().unwrap().container = Arc::downgrade(&ret);

        ret
    }
    pub fn init_ui() -> io::Result<()> {
        TermManager::init_term()?;
        Ok(())
    }

    pub fn start_page_ui(&self) -> io::Result<()> {
        StyleManager::set_foreground_color(Color::Cyan)?;
        let mut core = self.core.lock().unwrap();
        core.cursor
            .write_with_pos("Held - DragonOS/Linux Term Editor\n", 5, 0, false)?;
        StyleManager::set_foreground_color(Color::Green)?;
        core.cursor
            .write_with_pos("Author: heyicong@dragonos.org\n", 7, 1, false)?;
        StyleManager::set_foreground_color(Color::DarkMagenta)?;
        core.cursor
            .write_with_pos("Type any key to continue ><\n", 8, 2, false)?;
        StyleManager::reset_color()?;

        core.cursor.move_to(0, 0)?;

        #[cfg(feature = "dragonos")]
        let _ = Input::wait_keydown();

        #[cfg(not(feature = "dragonos"))]
        loop {
            let ev = crossterm::event::read()?;
            if let crossterm::event::Event::Key(_) = ev {
                break;
            }
        }

        TermManager::clear_all()?;

        Ok(())
    }

    pub fn ui_loop(&self) -> io::Result<bool> {
        let mut core = self.core.lock().unwrap();
        core.render_content(0, CONTENT_WINSIZE.read().unwrap().rows as usize)?;
        core.update_bottom_state_bar()?;
        core.cursor.move_to(0, 0)?;
        core.cursor.highlight(None)?;
        loop {
            #[cfg(feature = "dragonos")]
            let callback = {
                let key = Input::wait_keydown()?;
                self.mode.read().unwrap().key_event_route(&mut core, key)?
            };

            #[cfg(not(feature = "dragonos"))]
            let callback = {
                let ev = crossterm::event::read()?;
                self.mode.read().unwrap().event_route(&mut core, ev)?
            };

            match callback {
                super::event::WarpUiCallBackType::ChangMode(mode) => {
                    self.set_mode(mode, &mut core)?
                }
                super::event::WarpUiCallBackType::None => {}
                super::event::WarpUiCallBackType::Exit(store) => {
                    self.ui_exit();
                    return Ok(store);
                }
            }

            if self.mode.read().unwrap().mode_type() != ModeType::LastLine {
                core.update_bottom_state_bar()?;
            }
        }
    }

    fn set_mode(&self, mode: ModeType, ui: &mut MutexGuard<UiCore>) -> io::Result<()> {
        if mode != ModeType::LastLine {
            ui.cursor.set_prefix_mode(true);

            ui.render_content(0, CONTENT_WINSIZE.read().unwrap().rows as usize)?;
        }
        match mode {
            ModeType::Command => *self.mode.write().unwrap() = COMMAND.clone(),
            ModeType::LastLine => {
                ui.cursor.set_prefix_mode(false);
                let lastline = LASTLINE.clone();
                lastline.reset();
                *self.mode.write().unwrap() = lastline;

                ui.cursor.move_to(0, u16::MAX - 1)?;
                DEF_STYLE.read().unwrap().set_cmd_style()?;
                // 写一个空行
                ui.cursor.clear_current_line()?;
                ui.cursor.move_to_columu(0)?;
                ui.cursor.write(':')?;
            }
            ModeType::Insert => *self.mode.write().unwrap() = INSERT.clone(),
        }

        Ok(())
    }

    fn ui_exit(&self) {
        // 处理未保存退出时的提醒
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UiStyle {
    pub content_fg: Option<Color>,
    pub content_bg: Option<Color>,
    pub cmd_line_fg: Option<Color>,
    pub cmd_line_bg: Option<Color>,
}

impl UiStyle {
    pub fn set_cmd_style(&self) -> io::Result<()> {
        StyleManager::reset_color()?;
        if self.cmd_line_bg.is_some() {
            StyleManager::set_background_color(self.cmd_line_bg.unwrap())?;
        }
        if self.cmd_line_fg.is_some() {
            StyleManager::set_foreground_color(self.cmd_line_fg.unwrap())?;
        }

        Ok(())
    }

    pub fn set_content_style(&self) -> io::Result<()> {
        StyleManager::reset_color()?;
        if self.content_bg.is_some() {
            StyleManager::set_background_color(self.content_bg.unwrap())?;
        }
        if self.content_fg.is_some() {
            StyleManager::set_foreground_color(self.content_fg.unwrap())?;
        }

        Ok(())
    }
}
