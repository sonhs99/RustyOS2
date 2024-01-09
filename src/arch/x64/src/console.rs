use core::fmt;
use spin::{Lazy, Mutex};

use crate::{assembly::OutPortByte, keyboard};

const VGA_PORT_INDEX: u16 = 0x3D4;
const VGA_PORT_DATA: u16 = 0x3D5;
const VGA_INDEX_UPPERCURSER: u8 = 0x0E;
const VGA_INDEX_LOWERCURSER: u8 = 0x0F;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [ScreenChar; BUFFER_WIDTH * BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= (BUFFER_WIDTH * BUFFER_HEIGHT) {
                    self.new_line();
                }
                self.buffer.chars[self.column_position] = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
        self.set_curser(
            self.column_position % BUFFER_WIDTH,
            self.column_position / BUFFER_WIDTH,
        );
    }

    fn new_line(&mut self) {
        if self.column_position <= BUFFER_WIDTH * (BUFFER_HEIGHT - 1) - 1 {
            self.column_position += BUFFER_WIDTH - (self.column_position % BUFFER_WIDTH);
            return;
        }
        self.column_position -= BUFFER_WIDTH;
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row * BUFFER_WIDTH + col];
                self.buffer.chars[(row - 1) * BUFFER_WIDTH + col] = character;
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = (BUFFER_HEIGHT - 1) * BUFFER_WIDTH;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row * BUFFER_WIDTH + col] = blank;
        }
    }

    pub fn clear_screen(&mut self) {
        for i in 0..(BUFFER_HEIGHT * BUFFER_WIDTH) {
            self.buffer.chars[i] = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            }
        }
        self.set_curser(0, 0);
    }

    pub fn set_curser(&mut self, x: usize, y: usize) {
        let linear = y * BUFFER_WIDTH + x;
        OutPortByte(VGA_PORT_INDEX, VGA_INDEX_UPPERCURSER);
        OutPortByte(VGA_PORT_DATA, (linear >> 8) as u8);

        OutPortByte(VGA_PORT_INDEX, VGA_INDEX_LOWERCURSER);
        OutPortByte(VGA_PORT_DATA, (linear & 0xFF) as u8);

        self.column_position = linear;
    }

    pub fn get_curser(&self) -> (usize, usize) {
        (
            self.column_position % BUFFER_WIDTH,
            self.column_position / BUFFER_WIDTH,
        )
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub static WRITER: Lazy<Mutex<Writer>> = Lazy::new(|| {
    Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    })
});

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn clear_screen() {
    WRITER.lock().clear_screen();
}

pub fn getch() -> u8 {
    let mut key_data: keyboard::KeyData = keyboard::KeyData::new();
    loop {
        while !keyboard::GetKeyFromKeyQueue(&mut key_data) {}
        if (key_data.Flags & keyboard::KeyStatement::KeyFlagsDown as u8) != 0 {
            return key_data.ASCIICode;
        }
    }
}

pub fn set_curser(x: usize, y: usize) {
    WRITER.lock().set_curser(x, y);
}

pub fn get_curser() -> (usize, usize) {
    return WRITER.lock().get_curser();
}

pub fn init_console(x: usize, y: usize) {
    set_curser(x, y)
}
