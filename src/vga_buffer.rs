use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

/// Color used in the print! macros
pub const NORMAL_COLOR: ColorCode = ColorCode((Color::Black as u8) << 4 | (Color::White as u8));
/// Color used in the eprintln! macro
pub const ERR_COLOR: ColorCode  = ColorCode((Color::White as u8) << 4 | (Color::Red as u8));

/// Memory address at which we write output
const VGA_ADDRESS : usize = 0xb8000;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// global `Writer` used in the print! macros
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Color {
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

/// `ColorCode` is a u8 wrapper that contains
/// text color and background color
/// background color being the first 4 bits and
/// text color being the last 4 bits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl ColorCode {
    #![allow(dead_code)]
    fn new(text: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (text as u8))
    }
}


impl Writer {
    fn new() -> Self {
        Self { 
            column_position: 0, 
            color_code: NORMAL_COLOR,
            buffer: unsafe { &mut *(VGA_ADDRESS as *mut Buffer) }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // non printable ASCII so print '???'
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let c = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(c);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// global macros

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        $crate::vga_buffer::_set_print_color($crate::vga_buffer::ERR_COLOR);
        $crate::print!("{}", format_args!($($arg)*));
        $crate::vga_buffer::_set_print_color($crate::vga_buffer::NORMAL_COLOR);
        $crate::println!();
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}


#[doc(hidden)]
pub fn _set_print_color(color_code: ColorCode) {
    WRITER.lock().color_code = color_code;
}



#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}


#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        println!("\n{}", s);

        for (i, c) in  s.chars().enumerate() {
            let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}

#[test_case]
fn test_println_unsuported_chars() {
    use x86_64::instructions::interrupts;

    let s = "????????????????????";
    interrupts::without_interrupts(|| {
        println!("\n{}", s);

        for i in 0..(s.chars().count() * 2) {
            let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), char::from(0xfe));
        }
    });
}
