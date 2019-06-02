use std::char;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Content {
    Empty,
    Line(u8),
    Char(char),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Pixel {
    content: Content,
    colors: Option<Vec<Color>>,
}

impl Pixel {
    pub(crate) fn color(&self) -> Option<Color> {
        self.colors.as_ref().and_then(|colors| {
            if colors.is_empty() {
                return None;
            }
            let len = colors.len() as u32;
            let sum = colors.iter().fold((0, 0, 0), |acc, x| {
                (acc.0 + x.0 as u32, acc.1 + x.1 as u32, acc.2 + x.2 as u32)
            });
            Some(Color(
                (sum.0 / len) as u8,
                (sum.1 / len) as u8,
                (sum.2 / len) as u8,
            ))
        })
    }

    pub(crate) fn set_color(&mut self, color: Option<Color>) {
        self.colors = color.map(|color| vec![color]);
    }

    pub(crate) fn add_color(&mut self, color: Option<Color>) {
        if let Some(color) = color {
            self.colors.get_or_insert_with(|| Vec::new()).push(color)
        }
    }

    pub(crate) fn set_char(&mut self, c: char) {
        self.content = Content::Char(c);
    }

    pub(crate) fn unset_char(&mut self) {
        if let Content::Char(_) = self.content {
            self.content = Content::Empty;
            self.colors = None;
        }
    }

    pub(crate) fn set_line(&mut self, new: u8) {
        let old = match self.content {
            Content::Line(d) => d,
            _ => 0,
        };
        self.content = Content::Line(old | new);
    }

    pub(crate) fn unset_line(&mut self, new: u8) {
        if let Content::Line(ref mut d) = self.content {
            *d &= !new;
        };
    }

    pub(crate) fn toggle_line(&mut self, new: u8) {
        if let Content::Line(ref mut d) = self.content {
            *d ^= new;
        };
    }

    pub(crate) fn get_line(&self, dot: u8) -> bool {
        match self.content {
            Content::Line(d) => d & dot > 0,
            _ => false,
        }
    }

    pub(crate) fn get_char(&self, prev_color: Option<Color>) -> (String, Option<Color>) {
        let c = match self.content {
            Content::Empty => ' ',
            Content::Char(c) => c,
            Content::Line(d) => char::from_u32(0x2800 + d as u32).unwrap(),
        };
        let original_color = self.color();
        let (color, need_end) = if prev_color == original_color {
            (None, false)
        } else {
            (original_color.clone(), prev_color.is_some())
        };

        let mut s = String::new();
        if need_end {
            s.extend(colorize_char(None, None, true).chars());
        }
        s.extend(colorize_char(color, Some(c), false).chars());
        (s, original_color)
    }
}

impl Default for Pixel {
    fn default() -> Pixel {
        Pixel {
            content: Content::Empty,
            colors: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Color(u8, u8, u8);

impl Color {
    pub(crate) fn from_hex(hex: u32) -> Color {
        let (r, g, b) = (
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        );
        Color(r, g, b)
    }

    pub(crate) fn r(&self) -> u8 {
        self.0
    }

    pub(crate) fn g(&self) -> u8 {
        self.1
    }

    pub(crate) fn b(&self) -> u8 {
        self.2
    }
}

pub(crate) fn colorize_char(color: Option<Color>, c: Option<char>, append_end: bool) -> String {
    let mut s = String::new();
    if let Some(color) = color {
        s.push_str(&format!(
            "\x1B[38;2;{};{};{}m",
            color.r(),
            color.g(),
            color.b(),
        ));
    }
    if let Some(c) = c {
        s.push(c);
    }
    if append_end {
        s.push_str("\x1B[0m");
    }
    s
}
