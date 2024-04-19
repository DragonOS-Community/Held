use std::{io, sync::Arc};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::{
    config::appconfig::AppSetting,
    utils::{file::FileManager, ui::uicore::Ui},
};

pub struct Application {
    file_manager: FileManager,
    bak: bool,
    ui: Arc<Ui>,
}

impl Application {
    pub fn new(file_path: Option<String>, setting: AppSetting) -> io::Result<Self> {
        let bak;
        let mut file = if file_path.is_some() {
            bak = true;
            FileManager::new(file_path.unwrap())?
        } else {
            bak = false;
            FileManager::new("held.tmp".to_string())?
        };

        // 将文件数据读入buf
        let buf = file.init(bak)?;

        Ok(Self {
            file_manager: file,
            bak,
            ui: Ui::new(Arc::new(buf), setting),
        })
    }

    fn init(&mut self) -> io::Result<()> {
        Ui::init_ui()?;

        if !self.bak {
            self.ui.start_page_ui()?;
        }

        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        self.init()?;
        match self.ui.ui_loop() {
            Ok(store) => {
                if store {
                    let buffer = &self.ui.core.lock().unwrap().buffer;
                    self.file_manager.store(buffer)?
                } else if self.file_manager.is_first_open() {
                    self.file_manager.delete_files()?;
                }
            }
            Err(_) => {
                // 补救措施：恢复备份文件
                todo!()
            }
        }
        disable_raw_mode()?;
        Ok(())
    }
}
