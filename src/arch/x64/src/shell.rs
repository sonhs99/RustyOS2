use core::{hint::black_box, str};

use crate::{
    assembly::{read_TSC, DisableInterrupt, EnableInterrupt},
    console::{clear_screen, get_curser, getch, set_curser},
    keyboard::{KeySpecial, Reboot},
    print, print_string, println,
    process::{self, create_task, process_count, PRIORITY_HIGHIST, PRIORITY_LOWIST},
    timer::{convert_from_ms, init_PIT, wait, wait_using_PIT, Date, Time},
    utility::{get_ram_size, memset},
};

const CONSOLE_MAXCOMMANDBUFFERSIZE: usize = 300;
const CONSOLE_PROMPT: &'static str = ">";

type CommandFunc = fn(&mut Parameter);

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
        command: "wait",
        help: "Wait ms Using PIT",
        command_function: wait_PIT,
    },
    Command {
        command: "cpuspeed",
        help: "Measure CPU Speed",
        command_function: measure_cpu_speed,
    },
    Command {
        command: "date",
        help: "Show Current Date and time",
        command_function: show_date_and_time,
    },
    Command {
        command: "createtask",
        help: "Create Test Task",
        command_function: test_create_task,
    },
    Command {
        command: "checktask",
        help: "Check Task",
        command_function: check_task,
    },
    Command {
        command: "kill",
        help: "Kill Task",
        command_function: kill_task,
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
            let mut args = Parameter::new(args, " ");
            (command.command_function)(&mut args);
            break;
        }
    }
}

fn sHelp(_args: &mut Parameter) {
    println!("\n      ---   Shell Command List   ---\n");
    for command in COMMAND_TABLE {
        println!("{:10} {}", command.command, command.help);
    }
}
fn sCls(_args: &mut Parameter) {
    clear_screen();
}
fn sTotalRAMSize(_args: &mut Parameter) {
    println!("Total RAM Size: {} MB", get_ram_size());
}
fn kShutdown(_args: &mut Parameter) {
    println!("System Shutdown start...");
    println!("Press Any Key To Reboot PC");
    getch();
    Reboot();
}

fn show_date_and_time(_args: &mut Parameter) {
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

fn set_timer(args: &mut Parameter) {
    let count: u64 = match args.next() {
        Some(string) => match string.parse() {
            Ok(value) => value,
            Err(_) => {
                println!("settimer [ms] [0|1]");
                return;
            }
        },
        None => {
            println!("settimer [ms] [0|1]");
            return;
        }
    };
    let periodic: u64 = match args.next() {
        Some(string) => match string.parse() {
            Ok(value) => value,
            Err(_) => {
                println!("settimer [ms] [0|1]");
                return;
            }
        },
        None => {
            println!("settimer [ms] [0|1]");
            return;
        }
    };
    println!("{}", convert_from_ms(count));
    init_PIT(convert_from_ms(count) as u16, periodic > 0);
    println!(
        "Time = {} ms, Periodic = {}: Change Complete.",
        count,
        periodic > 0
    );
}

fn wait_PIT(args: &mut Parameter) {
    let milisecond: u64 = match args.next() {
        Some(string) => match string.parse() {
            Ok(value) => value,
            Err(_) => {
                println!("wait [ms]");
                return;
            }
        },
        None => {
            println!("wait [ms]");
            return;
        }
    };
    println!("{} ms Sleep Start...", milisecond);
    DisableInterrupt();
    wait(milisecond);
    EnableInterrupt();
    println!("{} ms Sleep Complete.", milisecond);
}

fn measure_cpu_speed(_args: &mut Parameter) {
    print!("Now Measuring");
    DisableInterrupt();
    let mut total = 0;
    for _ in 0..200 {
        let current = read_TSC();
        wait_using_PIT(convert_from_ms(50) as u16);
        total += read_TSC() - current;
        print!(".");
    }
    EnableInterrupt();
    println!("\nCPU Speed = {} MHz", total / 10 / 1000 / 1000);
}

fn test_task() {
    let mut i = 0;
    let offset = process::get_pid() * 2;
    let offset = 25 * 80 - (offset % (25 * 80));
    let data = [b'-', b'\\', b'|', b'/'];
    let vga = 0xb8000 as *mut u16;
    // println!(
    //     "[{:2X}] This is called from test_task, {}",
    //     process::get_pid(),
    //     i
    // );
    loop {
        // println!(
        //     "[{:2X}] This is called from test_task, {}",
        //     process::get_pid(),
        //     i
        // );
        let charactor = data[i % 4] as u16;
        let attribute = ((offset % 15) + 1) as u16;
        black_box(unsafe { *vga.offset(offset as isize) = charactor | attribute << 8 });
        i += 1;
    }
    // println!(
    //     "[{:2X}] This is called from test_task in the end, {}",
    //     process::get_pid(),
    //     i
    // );
}

fn test_create_task(args: &mut Parameter) {
    let count: u64 = match args.next() {
        Some(string) => match string.parse() {
            Ok(value) => value,
            Err(_) => {
                println!("createtask [count]");
                return;
            }
        },
        None => {
            println!("createtask [count]");
            return;
        }
    };
    for _ in 0..count {
        if let Err(_) = create_task(PRIORITY_HIGHIST, test_task as u64) {
            break;
        }
    }
}

fn check_task(_args: &mut Parameter) {
    println!("Total Task: {}", process_count());
}

fn kill_task(args: &mut Parameter) {
    let pid: u64 = match args.next() {
        Some(string) => match string.parse() {
            Ok(value) => value,
            Err(_) => {
                println!("kill [pid]");
                return;
            }
        },
        None => {
            println!("kill [pid]");
            return;
        }
    };
    if process::is_process_exist(pid) {
        println!("there are no Process [0x{pid:X}]");
    } else {
        process::end_process(pid);
    }
}

impl<'a> Parameter<'a> {
    pub fn new(args: &'a str, delim: &'a str) -> Self {
        Self { args, delim }
    }

    pub fn next(&mut self) -> Option<&'a str> {
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
