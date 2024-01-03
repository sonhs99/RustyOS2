use core::str;

use crate::{
    console::{clear_screen, get_curser, getch, set_curser},
    keyboard::{KeySpecial, Reboot},
    print, print_string, println,
    timer::{Date, Time},
    utility::{get_ram_size, memset},
};

const CONSOLE_MAXCOMMANDBUFFERSIZE: usize = 300;
const CONSOLE_PROMPT: &'static str = ">";

type CommandFunc = fn(&Parameter);

#[repr(C)]
struct Command {
    pub command: &'static str,
    pub help: &'static str,
    pub command_function: CommandFunc,
}
struct Parameter<'a> {
    delim: &'a str,
    args: &'a str,
}

static COMMAND_TABLE: &[Command] = &[
    Command {
        command: "help",
        help: "Show Help",
        command_function: sHelp,
    },
    Command {
        command: "cls",
        help: "Clear Screen",
        command_function: sCls,
    },
    Command {
        command: "totalram",
        help: "Show Total RAM Size",
        command_function: sTotalRAMSize,
    },
    Command {
        command: "shutdown",
        help: "Shutdown And Reboot OS",
        command_function: kShutdown,
    },
    Command {
        command: "settimer",
        help: "Set PIT Controller Counter0",
        command_function: set_timer,
    },
    Command {
        command: "date",
        help: "Show Current Date and time",
        command_function: show_date_and_time,
    },
];

pub fn start_shell() {
    let mut buffer: [u8; CONSOLE_MAXCOMMANDBUFFERSIZE] = [0u8; CONSOLE_MAXCOMMANDBUFFERSIZE];
    let mut buffer_index: usize = 0;
    print!("{}", CONSOLE_PROMPT);
    let mut key: u8;
    loop {
        key = getch();
        if key == KeySpecial::Backspace as u8 {
            if buffer_index > 0 {
                buffer_index -= 1;
                let curser = get_curser();
                print_string(curser.0 as i32 - 1, curser.1 as i32, b" ");
                set_curser(curser.0 - 1, curser.1);
            }
        } else if key == KeySpecial::Enter as u8 {
            print!("\n");
            if buffer_index > 0 {
                buffer[buffer_index] = b'\0';
                let command = str::from_utf8(&buffer).unwrap();
                let command = match command.split_once('\0') {
                    Some((com, _)) => com,
                    None => command,
                };
                execute_command(command);
            }
            print!("{}", CONSOLE_PROMPT);
            memset(
                buffer.as_mut_ptr(),
                b'\0',
                CONSOLE_MAXCOMMANDBUFFERSIZE as isize,
            );
            buffer_index = 0;
        } else if key == KeySpecial::Lshift as u8
            || key == KeySpecial::Rshift as u8
            || key == KeySpecial::CapsLock as u8
            || key == KeySpecial::NumLock as u8
            || key == KeySpecial::ScrollLock as u8
            || key == KeySpecial::Ctrl as u8
            || key == KeySpecial::Lalt as u8
        {
        } else {
            if key == KeySpecial::Tab as u8 {
                key = b' ';
            }
            if buffer_index < CONSOLE_MAXCOMMANDBUFFERSIZE {
                buffer[buffer_index] = key;
                buffer_index += 1;
            }
            print!("{}", str::from_utf8(&[key]).unwrap());
        }
    }
}

fn execute_command(buffer: &str) {
    let (com, args) = match buffer.split_once(' ') {
        Some((com, args)) => (com, args),
        None => (buffer, ""),
    };
    for command in COMMAND_TABLE {
        if command.command == com {
            let args = Parameter::new(args, " ");
            (command.command_function)(&args);
            break;
        }
    }
}

fn sHelp(_args: &Parameter) {
    println!("\n      ---   Shell Command List   ---\n");
    for command in COMMAND_TABLE {
        println!("{:10} {}", command.command, command.help);
    }
}
fn sCls(_args: &Parameter) {
    clear_screen();
}
fn sTotalRAMSize(_args: &Parameter) {
    println!("Total RAM Size: {} MB", get_ram_size());
}
fn kShutdown(_args: &Parameter) {
    println!("System Shutdown start...");
    println!("Press Any Key To Reboot PC");
    getch();
    Reboot();
}

fn show_date_and_time(_args: &Parameter) {
    let date = Date::current();
    let time = Time::current();

    println!(
        "Date: {}/{:02}/{:02} {}",
        date.year as u16 + 2000,
        date.month,
        date.day_of_month,
        date.week_string()
    );
    println!(
        "Time: {:02}:{:02}:{:02}",
        time.hour, time.minute, time.second
    );
}

fn set_timer(args: &Parameter) {}

impl<'a> Parameter<'a> {
    pub fn new(args: &'a str, delim: &'a str) -> Self {
        Self { args, delim }
    }

    pub fn next(mut self) -> Option<&'a str> {
        if self.args == "" {
            return None;
        }
        let (arg, args) = match self.args.split_once(self.delim) {
            Some((arg, args)) => (arg, args),
            None => (self.args, ""),
        };
        self.args = args;
        return Some(arg);
    }
}
