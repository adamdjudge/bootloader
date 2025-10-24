use core::fmt;
use core::mem;

use crate::port;

/// Console width in characters.
pub const WIDTH: usize = 80;
/// Console height in characters.
pub const HEIGHT: usize = 25;
/// The total console size in characters, defined as `WIDTH * HEIGHT`.
pub const SIZE: usize = WIDTH * HEIGHT;

/// Text color for characters and background shown on the console.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Color {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    Gray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    Pink,
    Yellow,
    White,
}

impl TryFrom<u8> for Color {
    type Error = u8;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0..16 => Ok(unsafe { mem::transmute(val) }),
            _ => Err(val),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
struct VgaChar {
    char: u8,
    attrs: u8,
}

#[repr(transparent)]
struct VgaBuffer {
    chars: [VgaChar; SIZE],
}

impl VgaBuffer {
    fn get() -> &'static mut Self {
        unsafe { &mut *(0xb8000 as *mut Self) }
    }

    fn update_cursor(position: usize) {
        unsafe {
            port::out8(0x3d4, 0xf);
            port::out8(0x3d5, (position & 0xff) as u8);
            port::out8(0x3d4, 0xe);
            port::out8(0x3d5, ((position >> 8) & 0xff) as u8);
        }
    }
}

/// Handle used for writing text to the VGA console.
pub struct Writer {
    position: usize,
    attrs: u8,
}

static mut WRITER: Writer = Writer {
    position: 0,
    attrs: Color::LightGray as u8,
};

#[allow(dead_code)]
impl Writer {
    /// Returns a mutable reference to the global console writer.
    pub fn get() -> &'static mut Self {
        unsafe { &mut *(&raw mut WRITER) }
    }

    /// Sets the current position of the console cursor. Returns `Ok` if the given cursor position
    /// is within bounds (`pos < SIZE`), otherwise returns `Err(pos)` and has no effect.
    pub fn set_position(&mut self, pos: usize) -> Result<(), usize> {
        match pos {
            0..SIZE => {
                self.position = pos;
                VgaBuffer::update_cursor(pos);
                Ok(())
            }
            _ => Err(pos),
        }
    }

    /// Returns the current position of the console cursor.
    pub fn get_position(&self) -> usize {
        self.position
    }

    /// Sets the text color for subsequent character writes to the console.
    pub fn set_text_color(&mut self, color: Color) {
        self.attrs = self.attrs & 0xf0 | color as u8;
    }

    /// Sets the background color for subsequent character writes to the console.
    pub fn set_bg_color(&mut self, color: Color) {
        self.attrs = self.attrs & 0x0f | (color as u8) << 4;
    }

    fn advance(&mut self, count: usize) {
        let pos = self.position + count;
        if pos < SIZE {
            self.position = pos;
        } else {
            self.position = WIDTH * (HEIGHT - 1);

            // Scroll text lines up, and then clear the bottom line
            let buffer = VgaBuffer::get();
            for line in 1..HEIGHT {
                let (prev, curr) = buffer.chars.split_at_mut(line * WIDTH);
                prev[(line - 1) * WIDTH..].clone_from_slice(&curr[..WIDTH]);
            }
            buffer.chars[(HEIGHT - 1) * WIDTH..].fill(VgaChar {
                char: 0,
                attrs: self.attrs,
            });
        }

        VgaBuffer::update_cursor(self.position);
    }

    fn put_byte(&mut self, b: u8) {
        VgaBuffer::get().chars[self.position] = VgaChar {
            char: b,
            attrs: self.attrs,
        };
        self.advance(1);
    }

    /// Clears the console by removing all text and setting the default colors.
    pub fn clear_screen(&mut self) {
        self.position = 0;
        self.set_text_color(Color::LightGray);
        self.set_bg_color(Color::Black);
        VgaBuffer::get().chars.fill(VgaChar {
            char: 0,
            attrs: self.attrs,
        });
    }

    /// Writes one character to the console.
    pub fn put_char(&mut self, c: char) {
        match c {
            '\0' => self.put_byte(0),
            '\n' => self.advance(WIDTH - self.position % WIDTH),
            ' '..='~' => self.put_byte(c as u8),
            '\u{80}'.. => self.put_byte(0xfe),
            _ => {}
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.put_char(c);
        }
        Ok(())
    }
}
