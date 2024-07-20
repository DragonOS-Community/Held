use std::{
    fs::{self, File},
    io::{self, Read, Seek, Write},
};

use super::buffer::EditBuffer;

use std::path::PathBuf;

pub const BAK_SUFFIX: &'static str = ".heldbak";

pub struct FileManager {
    name: String,
    file: File,
    is_first_open: bool,
    bak: Option<File>,
}

impl FileManager {
    pub fn new(file_path: String) -> io::Result<Self> {
        let ifo_flag = !PathBuf::from(file_path.clone()).exists();

        let file = File::options()
            .write(true)
            .read(true)
            .create(true)
            .open(file_path.clone())?;

        Ok(Self {
            file,
            is_first_open: ifo_flag,
            name: file_path,
            bak: None,
        })
    }

    pub fn init(&mut self, bak: bool) -> io::Result<EditBuffer> {
        let mut buf = Vec::new();
        // 是否备份
        if bak {
            self.do_bak(&mut buf)?;
        } else {
            self.file.read_to_end(&mut buf)?;
        }

        Ok(EditBuffer::new(buf))
    }

    // 备份
    fn do_bak(&mut self, buf: &mut Vec<u8>) -> io::Result<()> {
        let mut bak = File::options()
            .write(true)
            .read(true)
            .create(true)
            .open(format!("{}{}", self.name, BAK_SUFFIX))?;

        bak.set_len(0)?;

        self.file.read_to_end(buf)?;
        bak.write_all(&buf)?;

        self.file.seek(io::SeekFrom::Start(0))?;

        if self.bak.is_some() {
            error!("The backup already exists. The operation may cause data loss.");
        }

        self.bak = Some(bak);

        Ok(())
    }

    pub fn store(&mut self, buf: &EditBuffer) -> io::Result<()> {
        let data = buf.all_buffer();

        self.file.set_len(0)?;

        for (idx, line) in data.iter().enumerate() {
            if idx == data.len() - 1 {
                self.file.write(&line[..line.len()])?;
            } else {
                self.file.write(&line)?;
            }
        }

        if self.bak.is_some() {
            fs::remove_file(format!("{}{}", self.name, BAK_SUFFIX))?;
        }
        self.is_first_open = false;

        Ok(())
    }

    pub fn is_first_open(&mut self) -> bool {
        self.is_first_open
    }

    pub fn delete_files(&mut self) -> io::Result<()> {
        if !self.name.is_empty() {
            fs::remove_file(self.name.clone())?;
        }

        if self.bak.is_some() {
            fs::remove_file(format!("{}{}", self.name, BAK_SUFFIX))?;
        }
        Ok(())
    }
}
