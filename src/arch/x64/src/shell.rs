use crate::{console::{get_curser, getch, set_curser}, keyboard::KeySpecial, print, print_string, utility::memset};

const CONSOLE_MAXCOMMANDBUFFERSIZE: usize = 300;
const CONSOLE_PROMPT: &'static str = ">";

type CommandFunc = fn(&[u8]);

#[repr(C, packed(1))]
struct Command {
    pub command: &'static str,
    pub help: &'static str,
    pub command_function: CommandFunc
}
struct Parameter {

}

// static COMMAND_TABLE: &[Command] = &[
//     Command{ command:"help", help: "Show Help", command_function:sHelp },
//     Command{ command:"cls", help:"Clear Screen", command_function:sCls },
//     Command{ command:"totalram", help:"Show Total RAM Size", command_function:sTotalRAMSize },
//     Command{ command:"shutdown", help:"Show Total RAM Size", command_function:kShutdown },
// ];

pub fn start_shell(){
    // let mut buffer: [u8; CONSOLE_MAXCOMMANDBUFFERSIZE] = [0u8; CONSOLE_MAXCOMMANDBUFFERSIZE];
    // let mut buffer_index: usize = 0;
    print!("{}", CONSOLE_PROMPT);
    // let mut key: u8;
    loop {
        // key = getch();
        // if key == KeySpecial::Backspace as u8 {
        //     if buffer_index > 0 {
        //         let curser = get_curser();
        //         print_string(curser.0 as i32 - 1, curser.1 as i32, b" ");
        //         set_curser(curser.0 - 1, curser.1);
        //         buffer_index -= 1;
        //     }
        // } else if key == KeySpecial::Enter as u8 {
        //     print!("\n");
        //     if buffer_index > 0 {
        //         // execute_command(buffer[..buffer_index]);
        //     }
        //     print!("{}", CONSOLE_PROMPT);
        //     memset(buffer.as_mut_ptr(), b'\0', CONSOLE_MAXCOMMANDBUFFERSIZE as isize);
        // } else if key == KeySpecial::Lshift as u8 || key == KeySpecial::Rshift as u8
        //         || key == KeySpecial::CapsLock as u8 || key == KeySpecial::NumLock as u8 || key == KeySpecial::ScrollLock as u8 {
        // } else {
        //     if key == KeySpecial::Tab as u8 { key = b' '; }
        //     if buffer_index < CONSOLE_MAXCOMMANDBUFFERSIZE {
        //         buffer[buffer_index] = key;
        //         print!("{}", key as char);
        //     }
        // }
    }
}

