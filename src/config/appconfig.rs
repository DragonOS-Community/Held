use crossterm::style::Color;

#[derive(Debug, serde::Deserialize, Default)]
pub struct DeserializeAppOption {
    pub line: Option<DeserializeLineOption>,
}

#[derive(Debug)]
pub struct AppSetting {
    pub line: LineSetting,
}

#[derive(Debug, serde::Deserialize, Clone, Copy)]
pub struct DeserializeLineOption {
    pub number: Option<DeserializeLineNumber>,
    pub highlight: Option<DeserializeHighLightSetting>,
}

#[derive(Debug, Clone, Copy)]
pub struct LineSetting {
    pub line_num: LineNumberSetting,
    pub highlight: HighLightSetting,

    pub prefix_width: usize,
}

#[derive(Debug, serde::Deserialize, Clone, Copy)]
pub struct DeserializeLineNumber {
    pub enable: bool,
    pub background: Option<u32>,
    pub frontground: Option<u32>,
}

impl DeserializeLineNumber {
    pub fn to_line_number_setting(setting: Option<Self>) -> LineNumberSetting {
        let mut ret = LineNumberSetting::default();
        if setting.is_none() {
            return ret;
        } else {
            let setting = setting.unwrap();
            if setting.background.is_some() {
                let color = setting.background.unwrap();
                let r = (color & 0xff0000) >> 16;
                let g = (color & 0x00ff00) >> 8;
                let b = color & 0x0000ff;

                ret.background = Color::Rgb {
                    r: r as u8,
                    g: g as u8,
                    b: b as u8,
                }
            }

            if setting.frontground.is_some() {
                let color = setting.frontground.unwrap();
                let r = (color & 0xff0000) >> 16;
                let g = (color & 0x00ff00) >> 8;
                let b = color & 0x0000ff;

                ret.frontground = Color::Rgb {
                    r: r as u8,
                    g: g as u8,
                    b: b as u8,
                }
            }

            return ret;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LineNumberSetting {
    pub enable: bool,
    pub background: Color,
    pub frontground: Color,
}

impl Default for LineNumberSetting {
    fn default() -> Self {
        Self {
            enable: true,
            background: Color::DarkGreen,
            frontground: Color::White,
        }
    }
}

impl Default for LineSetting {
    fn default() -> Self {
        Self {
            line_num: LineNumberSetting::default(),
            highlight: Default::default(),
            prefix_width: 0,
        }
    }
}

// 高亮选项
#[derive(Debug, serde::Deserialize, Clone, Copy)]
pub struct DeserializeHighLightSetting {
    pub enable: bool,
    pub color: u32,
}

impl DeserializeHighLightSetting {
    pub fn to_highlight_setting(&self) -> HighLightSetting {
        let r = (self.color & 0xff0000) >> 16;
        let g = (self.color & 0x00ff00) >> 8;
        let b = self.color & 0x0000ff;

        HighLightSetting {
            enable: self.enable,
            color: Color::Rgb {
                r: r as u8,
                g: g as u8,
                b: b as u8,
            },
        }
    }
}

// 高亮选项
#[derive(Debug, Clone, Copy)]
pub struct HighLightSetting {
    pub enable: bool,
    pub color: Color,
}

impl Default for HighLightSetting {
    fn default() -> Self {
        Self {
            enable: true,
            color: Color::DarkYellow,
        }
    }
}

impl DeserializeAppOption {
    pub fn to_app_setting(&self) -> AppSetting {
        let line_setting = match self.line {
            Some(setting) => setting.to_line_setting(),
            None => LineSetting::default(),
        };

        AppSetting { line: line_setting }
    }
}

impl DeserializeLineOption {
    pub fn to_line_setting(&self) -> LineSetting {
        let mut highlight = HighLightSetting::default();
        if self.highlight.is_some() {
            let h = self.highlight.unwrap();
            highlight = h.to_highlight_setting();
        }
        LineSetting {
            line_num: DeserializeLineNumber::to_line_number_setting(self.number),
            highlight,
            prefix_width: 0,
        }
    }
}
