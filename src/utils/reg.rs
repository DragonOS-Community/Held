use super::buffer::LineBuffer;

#[derive(Debug)]
pub struct Register {
    pub text: Vec<LineBuffer>
}

impl Register {
    pub fn is_muti_line(&self) -> bool {
        return self.text.len() > 1;
    }
    
    pub fn is_single_line(&self) -> bool {
        return self.text.len() == 1 && self.text[0].ends_with(b"\n");
    }

    pub fn new() -> Register {
        Register {
            text: Vec::new()
        }
    }

    pub fn from(lines: Vec<LineBuffer>) -> Register {
        Register {
            text: Vec::from(lines)
        }
    }

    pub fn copy(&mut self, lines: Vec<LineBuffer>) {
        self.text = Vec::from(lines);
    }
}