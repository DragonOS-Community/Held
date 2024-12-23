use std::{collections::HashMap, sync::MutexGuard};

use lazy_static::lazy_static;

use crate::utils::{
    buffer::LineState,
    ui::{
        event::WarpUiCallBackType,
        uicore::{UiCore, APP_INTERNAL_INFOMATION, CONTENT_WINSIZE},
        InfoLevel,
    },
};

lazy_static! {
    static ref COMMAND: HashMap<&'static str, fn(&mut MutexGuard<UiCore>, &str) -> WarpUiCallBackType> = {
        let mut map: HashMap<&str, fn(&mut MutexGuard<UiCore>, &str) -> WarpUiCallBackType> =
            HashMap::new();
        map.insert(":q!", LastLineCommand::force_exit);
        map.insert(":q", LastLineCommand::exit_without_store);
        map.insert(":wq", LastLineCommand::exit);

        // 跳转
        map.insert(":goto", LastLineCommand::goto);
        map.insert(":gt", LastLineCommand::goto);

        // 标记或锁定
        map.insert(":flag", LastLineCommand::flag);
        map.insert(":lock", LastLineCommand::lock);
        map.insert(":unflag", LastLineCommand::unflag);
        map.insert(":unlock", LastLineCommand::unlock);

        map.insert(":delete", LastLineCommand::delete_lines);
        map.insert(":dl", LastLineCommand::delete_lines);

        map
    };
}

const EDITED_NO_STORE: &'static str = "Changes have not been saved";
const NOT_FOUNT_CMD: &'static str = "Command Not Fount";

#[derive(Debug)]
#[allow(unused)]
pub struct LastLineCommand {
    /// Command
    pub command: String,
    pub args: Vec<String>,
}

/// 提供给用户的命令行功能
impl LastLineCommand {
    pub fn process(ui: &mut MutexGuard<UiCore>, cmd: String) -> WarpUiCallBackType {
        let args = cmd
            .splitn(2, |x: char| x.is_ascii_whitespace())
            .collect::<Vec<_>>();

        if let Some(func) = COMMAND.get(args[0]) {
            let ret = if args.len() == 1 {
                func(ui, "")
            } else {
                func(ui, &args[1])
            };

            ret
        } else {
            let mut info = APP_INTERNAL_INFOMATION.lock().unwrap();
            info.level = InfoLevel::Info;
            info.info = NOT_FOUNT_CMD.to_string();
            return WarpUiCallBackType::None;
        }
    }

    const fn is_split_char(x: char) -> bool {
        x == ',' || x == ';' || x == ':' || x == '/' || x.is_ascii_whitespace()
    }

    fn force_exit(_ui: &mut MutexGuard<UiCore>, _args: &str) -> WarpUiCallBackType {
        WarpUiCallBackType::Exit(false)
    }

    fn exit_without_store(ui: &mut MutexGuard<UiCore>, _args: &str) -> WarpUiCallBackType {
        if ui.edited() {
            // 编辑过但不保存？
            // 更新警示信息
            let mut info = APP_INTERNAL_INFOMATION.lock().unwrap();
            info.level = InfoLevel::Warn;
            info.info = EDITED_NO_STORE.to_string();
            return WarpUiCallBackType::None;
        }
        WarpUiCallBackType::Exit(false)
    }

    fn exit(_ui: &mut MutexGuard<UiCore>, _args: &str) -> WarpUiCallBackType {
        WarpUiCallBackType::Exit(true)
    }

    fn goto(ui: &mut MutexGuard<UiCore>, args: &str) -> WarpUiCallBackType {
        if args.is_empty() {
            let mut info = APP_INTERNAL_INFOMATION.lock().unwrap();
            info.level = InfoLevel::Info;
            info.info = "Useage: {goto}|{gt} {row}{' '|','|';'|':'|'/'}{col}".to_string();
            return WarpUiCallBackType::None;
        }
        let (y, x) = {
            let a = args.split(|x| Self::is_split_char(x)).collect::<Vec<_>>();

            if a.len() == 1 {
                (u16::from_str_radix(a[0], 10), Ok(1))
            } else {
                (u16::from_str_radix(a[0], 10), u16::from_str_radix(a[1], 10))
            }
        };

        if y.is_err() {
            let mut info = APP_INTERNAL_INFOMATION.lock().unwrap();
            info.level = InfoLevel::Info;
            info.info = "Useage: goto {row}({' '|','|';'|':'|'/'}{col})".to_string();
            return WarpUiCallBackType::None;
        }

        let buf_line_max = ui.buffer.line_count() as u16;
        let content_line_max = CONTENT_WINSIZE.read().unwrap().rows;
        let mut y = y.unwrap().min(buf_line_max);
        let mut x = x.unwrap_or(1).min(ui.buffer.get_linesize(y));

        if y == 0 {
            y += 1;
        }
        if x == 0 {
            x += 1;
        }

        x -= 1;
        y -= 1;

        ui.cursor.set_prefix_mode(true);

        ui.cursor.restore_pos().unwrap();

        // if y < ui.buffer.offset() as u16 + content_line_max {
        //     ui.buffer.set_offset(0);
        // } else {
        //     ui.buffer.set_offset((y - content_line_max) as usize);
        // }

        let lasty = ui.cursor.y();
        let y = ui.buffer.goto_line(y as usize);
        ui.cursor.move_to(x, y).unwrap();

        let pos = ui.cursor.store_tmp_pos();
        ui.render_content(0, content_line_max as usize).unwrap();
        ui.cursor.restore_tmp_pos(pos).unwrap();

        ui.cursor.highlight(Some(lasty)).unwrap();

        ui.cursor.store_pos();

        return WarpUiCallBackType::None;
    }

    // 标记行
    pub fn flag(ui: &mut MutexGuard<UiCore>, args: &str) -> WarpUiCallBackType {
        let args = args.split(|x| Self::is_split_char(x)).collect::<Vec<_>>();

        if args.len() == 0 {
            ui.buffer
                .add_line_flags(ui.cursor.cmd_y() as usize - 1, LineState::FLAGED);
        }

        for s in args {
            let line = usize::from_str_radix(s, 10);
            if line.is_err() {
                APP_INTERNAL_INFOMATION.lock().unwrap().info = format!("\"{s}\" is not a number");
                return WarpUiCallBackType::None;
            }

            let line = line.unwrap();
            ui.buffer.add_line_flags(line - 1, LineState::FLAGED);
        }

        WarpUiCallBackType::None
    }

    // 锁定行
    pub fn lock(ui: &mut MutexGuard<UiCore>, args: &str) -> WarpUiCallBackType {
        let args = args.split(|x| Self::is_split_char(x)).collect::<Vec<_>>();

        match args.len() {
            0 => {
                //没有参数，锁定当前行
                ui.buffer
                    .add_line_flags(ui.cursor.cmd_y() as usize - 1, LineState::LOCKED)
            }
            _ => {
                //有参数，锁定指定行
                for arg in args {
                    let line = usize::from_str_radix(arg, 10);
                    if line.is_err() {
                        APP_INTERNAL_INFOMATION.lock().unwrap().info =
                            format!("\"{arg}\" is not a number");
                        return WarpUiCallBackType::None;
                    }

                    let line = line.unwrap();
                    ui.buffer.add_line_flags(line - 1, LineState::LOCKED);
                }
            }
        }

        WarpUiCallBackType::None
    }

    // 标记行
    pub fn unflag(ui: &mut MutexGuard<UiCore>, args: &str) -> WarpUiCallBackType {
        let args = args.split(|x| Self::is_split_char(x)).collect::<Vec<_>>();

        match args.len() {
            0 => {
                //没有参数，解除标记当前行
                ui.buffer
                    .remove_line_flags(ui.cursor.cmd_y() as usize - 1, LineState::FLAGED)
            }
            _ => {
                //有参数，解除标记指定行
                for arg in args {
                    let line = usize::from_str_radix(arg, 10);
                    if line.is_err() {
                        APP_INTERNAL_INFOMATION.lock().unwrap().info =
                            format!("\"{arg}\" is not a number");
                        return WarpUiCallBackType::None;
                    }

                    let line = line.unwrap();
                    ui.buffer.remove_line_flags(line - 1, LineState::FLAGED);
                }
            }
        }

        WarpUiCallBackType::None
    }

    // 解除锁定行
    pub fn unlock(ui: &mut MutexGuard<UiCore>, args: &str) -> WarpUiCallBackType {
        let args = args.split(|x| Self::is_split_char(x)).collect::<Vec<_>>();

        match args.len() {
            0 => {
                //没有参数，解除锁定当前行
                ui.buffer
                    .remove_line_flags(ui.cursor.cmd_y() as usize - 1, LineState::LOCKED)
            }
            _ => {
                //有参数，解除锁定指定行
                for arg in args {
                    let line = usize::from_str_radix(arg, 10);
                    if line.is_err() {
                        APP_INTERNAL_INFOMATION.lock().unwrap().info =
                            format!("\"{arg}\" is not a number");
                        return WarpUiCallBackType::None;
                    }

                    let line = line.unwrap();
                    ui.buffer.remove_line_flags(line - 1, LineState::LOCKED);
                }
            }
        }

        WarpUiCallBackType::None
    }

    pub fn delete_lines(ui: &mut MutexGuard<UiCore>, args: &str) -> WarpUiCallBackType {
        let args = args.split(|x| x == '-').collect::<Vec<_>>();

        match args.len() {
            0 => {
                let offset = ui.buffer.offset() + ui.cursor.y() as usize;
                let count = ui.buffer.delete_lines(offset, offset + 1);
                if count != 0 {
                    APP_INTERNAL_INFOMATION.lock().unwrap().info =
                        format!("Successfully deleted {count} row");
                }
                ui.render_content(0, CONTENT_WINSIZE.read().unwrap().rows as usize)
                    .unwrap();
                return WarpUiCallBackType::None;
            }
            1 => {
                let line = usize::from_str_radix(args[0], 10);
                if line.is_err() {
                    APP_INTERNAL_INFOMATION.lock().unwrap().info =
                        format!("\"{}\" is not a number", args[0]);
                    return WarpUiCallBackType::None;
                }

                let line = line.unwrap();

                let offset = ui.buffer.offset() + line;
                let count = ui.buffer.delete_lines(offset, offset + 1);
                if count != 0 {
                    APP_INTERNAL_INFOMATION.lock().unwrap().info =
                        format!("Successfully deleted {count} row");
                }
                ui.render_content(0, CONTENT_WINSIZE.read().unwrap().rows as usize)
                    .unwrap();
                return WarpUiCallBackType::None;
            }
            _ => {
                let start = usize::from_str_radix(args[0], 10);
                let end = usize::from_str_radix(args[1], 10);

                if start.is_err() || end.is_err() {
                    APP_INTERNAL_INFOMATION.lock().unwrap().info =
                        "Useage: (dl)|(delete) {start}({'-'}{end})".to_string();
                    return WarpUiCallBackType::None;
                }

                let count = ui.buffer.delete_lines(start.unwrap() - 1, end.unwrap() - 1);
                if count != 0 {
                    APP_INTERNAL_INFOMATION.lock().unwrap().info =
                        format!("Successfully deleted {count} row");
                }

                ui.render_content(0, CONTENT_WINSIZE.read().unwrap().rows as usize)
                    .unwrap();
            }
        }
        return WarpUiCallBackType::None;
    }
}
