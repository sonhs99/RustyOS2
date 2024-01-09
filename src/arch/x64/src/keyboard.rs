#![allow(non_snake_case)]
#![allow(dead_code)]

use spin::{Lazy, Mutex};

use crate::{
    assembly::{InPortByte, OutPortByte},
    println,
    types::StaticQueue,
    utility::set_interrupt_flag,
};

const KEY_SKIPCOUNTFORPAUSE: i32 = 2;
const KEY_MAPPINGTABLEMAXCOUNT: usize = 89;
const KEY_MAXQUEUECOUNT: usize = 100;
pub enum KeyStatement {
    KeyFlagsUp = 0x00,
    KeyFlagsDown = 0x01,
    KeyFlagsExtendedkey = 0x02,
}

pub enum KeySpecial {
    None = 0x00,
    Enter = '\n' as isize,
    Tab = '\t' as isize,
    Esc = 0x1B,
    Backspace = 0x08,
    Ctrl = 0x81,
    Lshift = 0x82,
    Rshift = 0x83,
    PrintScreen = 0x84,
    Lalt = 0x85,
    CapsLock = 0x86,
    F1 = 0x87,
    F2 = 0x88,
    F3 = 0x89,
    F4 = 0x8A,
    F5 = 0x8B,
    F6 = 0x8C,
    F7 = 0x8D,
    F8 = 0x8E,
    F9 = 0x8F,
    F10 = 0x90,
    NumLock = 0x91,
    ScrollLock = 0x92,
    Home = 0x93,
    Up = 0x94,
    PageUp = 0x95,
    Left = 0x96,
    Center = 0x97,
    Right = 0x98,
    End = 0x99,
    Down = 0x9A,
    PageDown = 0x9B,
    Insert = 0x9C,
    Delete = 0x9D,
    F11 = 0x9E,
    F12 = 0x9F,
    Pause = 0xA0,
}

#[repr(packed(1))]
#[derive(Clone, Copy)]
pub struct KeyData {
    pub ScanCode: u8,
    pub ASCIICode: u8,
    pub Flags: u8,
}

impl KeyData {
    pub fn new() -> Self {
        Self {
            ScanCode: 0,
            ASCIICode: 0,
            Flags: 0,
        }
    }
}

#[repr(packed(1))]
struct KeyMappingEntry(u8, u8);

impl KeyMappingEntry {
    pub fn NormalCode(&self) -> u8 {
        self.0
    }
    pub fn CombinedCode(&self) -> u8 {
        self.1
    }
}
struct KeyboardManager {
    ShiftDown: bool,
    CapsLockOn: bool,
    NumLockOn: bool,
    ScrollLockOn: bool,

    ExtendedCodeIn: bool,
    SkipCountForPause: i32,
}

pub fn IsOutputBufferFull() -> bool {
    (InPortByte(0x64) & 0x01) != 0
}

pub fn IsInputBufferFull() -> bool {
    (InPortByte(0x64) & 0x02) != 0
}

pub fn WaitForACKAndPutOtherScanCode() -> bool {
    let mut result = false;
    for _ in 0..1000 {
        for _ in 0..0xFFFF {
            if IsOutputBufferFull() {
                break;
            }
        }
        let data = InPortByte(0x60);
        if data == 0xFA {
            result = true;
            break;
        } else {
            ConvertScanCodeAndPutQueue(data);
        }
    }
    result
}

pub fn ActiveKeyboard() -> bool {
    let previous_interrupt = set_interrupt_flag(false);
    OutPortByte(0x64, 0xAE);
    for _ in 0..0xFFFF {
        if !IsInputBufferFull() {
            break;
        }
    }
    OutPortByte(0x60, 0xF4);

    let result = WaitForACKAndPutOtherScanCode();
    set_interrupt_flag(previous_interrupt);
    result
}

pub fn GetKeyboardScanCode() -> u8 {
    while !IsOutputBufferFull() {}
    return InPortByte(0x60);
}

pub fn ChangeKeyboardLED(CapsLockOn: bool, NumLockOn: bool, ScrollLockOn: bool) -> bool {
    let previous_interrupt = set_interrupt_flag(false);
    for _ in 0..0xFFFF {
        if !IsInputBufferFull() {
            break;
        }
    }
    OutPortByte(0x60, 0xED);
    for _ in 0..0xFFFF {
        if !IsInputBufferFull() {
            break;
        }
    }

    let result = WaitForACKAndPutOtherScanCode();

    if result {
        set_interrupt_flag(previous_interrupt);
        return false;
    }

    OutPortByte(
        0x60,
        ((CapsLockOn as u8) << 2) | ((NumLockOn as u8) << 1) | (ScrollLockOn as u8),
    );

    for _ in 0..0xFFFF {
        if !IsInputBufferFull() {
            break;
        }
    }

    let result = WaitForACKAndPutOtherScanCode();
    set_interrupt_flag(previous_interrupt);
    return result;
}

pub fn EnableA20Gate() {
    OutPortByte(0x64, 0xD0);
    for _ in 0..0xFFFF {
        if IsOutputBufferFull() {
            break;
        }
    }

    let OutputPortData = InPortByte(0x60);
    let OutputPortData = OutputPortData | 0x01;

    for _ in 0..0xFFFF {
        if !IsInputBufferFull() {
            break;
        }
    }
    OutPortByte(0x64, 0xD1);
    OutPortByte(0x60, OutputPortData);
}

pub fn Reboot() {
    for _ in 0..0xFFFF {
        if !IsInputBufferFull() {
            break;
        }
    }
    OutPortByte(0x64, 0xD1);
    OutPortByte(0x60, 0x00);

    loop {}
}

static mut gs_stKeyboardManager: KeyboardManager = KeyboardManager {
    ShiftDown: false,
    CapsLockOn: false,
    NumLockOn: false,
    ScrollLockOn: false,

    ExtendedCodeIn: false,
    SkipCountForPause: 0,
};

static mut KeyBuffer: [KeyData; KEY_MAXQUEUECOUNT] = [KeyData {
    ScanCode: 0,
    ASCIICode: 0,
    Flags: 0,
}; 100];

static mut KeyQueue: Lazy<Mutex<StaticQueue<KeyData>>> = Lazy::new(|| {
    Mutex::new(StaticQueue::new(KEY_MAXQUEUECOUNT, unsafe {
        &mut KeyBuffer
    }))
});

static KeyMappingTable: [KeyMappingEntry; KEY_MAPPINGTABLEMAXCOUNT] = [
    /*  0   */ KeyMappingEntry(KeySpecial::None as u8, KeySpecial::None as u8),
    /*  1   */ KeyMappingEntry(KeySpecial::Esc as u8, KeySpecial::Esc as u8),
    /*  2   */ KeyMappingEntry('1' as u8, '!' as u8),
    /*  3   */ KeyMappingEntry('2' as u8, '@' as u8),
    /*  4   */ KeyMappingEntry('3' as u8, '#' as u8),
    /*  5   */ KeyMappingEntry('4' as u8, '$' as u8),
    /*  6   */ KeyMappingEntry('5' as u8, '%' as u8),
    /*  7   */ KeyMappingEntry('6' as u8, '^' as u8),
    /*  8   */ KeyMappingEntry('7' as u8, '&' as u8),
    /*  9   */ KeyMappingEntry('8' as u8, '*' as u8),
    /*  10  */ KeyMappingEntry('9' as u8, '(' as u8),
    /*  11  */ KeyMappingEntry('0' as u8, ')' as u8),
    /*  12  */ KeyMappingEntry('-' as u8, '_' as u8),
    /*  13  */ KeyMappingEntry('=' as u8, '+' as u8),
    /*  14  */ KeyMappingEntry(KeySpecial::Backspace as u8, KeySpecial::Backspace as u8),
    /*  15  */ KeyMappingEntry(KeySpecial::Tab as u8, KeySpecial::Tab as u8),
    /*  16  */ KeyMappingEntry('q' as u8, 'Q' as u8),
    /*  17  */ KeyMappingEntry('w' as u8, 'W' as u8),
    /*  18  */ KeyMappingEntry('e' as u8, 'E' as u8),
    /*  19  */ KeyMappingEntry('r' as u8, 'R' as u8),
    /*  20  */ KeyMappingEntry('t' as u8, 'T' as u8),
    /*  21  */ KeyMappingEntry('y' as u8, 'Y' as u8),
    /*  22  */ KeyMappingEntry('u' as u8, 'U' as u8),
    /*  23  */ KeyMappingEntry('i' as u8, 'I' as u8),
    /*  24  */ KeyMappingEntry('o' as u8, 'O' as u8),
    /*  25  */ KeyMappingEntry('p' as u8, 'P' as u8),
    /*  26  */ KeyMappingEntry('[' as u8, '{' as u8),
    /*  27  */ KeyMappingEntry(']' as u8, '}' as u8),
    /*  28  */ KeyMappingEntry('\n' as u8, '\n' as u8),
    /*  29  */ KeyMappingEntry(KeySpecial::Ctrl as u8, KeySpecial::Ctrl as u8),
    /*  30  */ KeyMappingEntry('a' as u8, 'A' as u8),
    /*  31  */ KeyMappingEntry('s' as u8, 'S' as u8),
    /*  32  */ KeyMappingEntry('d' as u8, 'D' as u8),
    /*  33  */ KeyMappingEntry('f' as u8, 'F' as u8),
    /*  34  */ KeyMappingEntry('g' as u8, 'G' as u8),
    /*  35  */ KeyMappingEntry('h' as u8, 'H' as u8),
    /*  36  */ KeyMappingEntry('j' as u8, 'J' as u8),
    /*  37  */ KeyMappingEntry('k' as u8, 'K' as u8),
    /*  38  */ KeyMappingEntry('l' as u8, 'L' as u8),
    /*  39  */ KeyMappingEntry(';' as u8, ':' as u8),
    /*  40  */ KeyMappingEntry('\'' as u8, '\"' as u8),
    /*  41  */ KeyMappingEntry('`' as u8, '~' as u8),
    /*  42  */ KeyMappingEntry(KeySpecial::Lshift as u8, KeySpecial::Lshift as u8),
    /*  43  */ KeyMappingEntry('\\' as u8, '|' as u8),
    /*  44  */ KeyMappingEntry('z' as u8, 'Z' as u8),
    /*  45  */ KeyMappingEntry('x' as u8, 'X' as u8),
    /*  46  */ KeyMappingEntry('c' as u8, 'C' as u8),
    /*  47  */ KeyMappingEntry('v' as u8, 'V' as u8),
    /*  48  */ KeyMappingEntry('b' as u8, 'B' as u8),
    /*  49  */ KeyMappingEntry('n' as u8, 'N' as u8),
    /*  50  */ KeyMappingEntry('m' as u8, 'M' as u8),
    /*  51  */ KeyMappingEntry(',' as u8, '<' as u8),
    /*  52  */ KeyMappingEntry('.' as u8, '>' as u8),
    /*  53  */ KeyMappingEntry('/' as u8, '?' as u8),
    /*  54  */ KeyMappingEntry(KeySpecial::Rshift as u8, KeySpecial::Rshift as u8),
    /*  55  */ KeyMappingEntry('*' as u8, '*' as u8),
    /*  56  */ KeyMappingEntry(KeySpecial::Lalt as u8, KeySpecial::Lalt as u8),
    /*  57  */ KeyMappingEntry(' ' as u8, ' ' as u8),
    /*  58  */ KeyMappingEntry(KeySpecial::CapsLock as u8, KeySpecial::CapsLock as u8),
    /*  59  */ KeyMappingEntry(KeySpecial::F1 as u8, KeySpecial::F1 as u8),
    /*  60  */ KeyMappingEntry(KeySpecial::F2 as u8, KeySpecial::F2 as u8),
    /*  61  */ KeyMappingEntry(KeySpecial::F3 as u8, KeySpecial::F3 as u8),
    /*  62  */ KeyMappingEntry(KeySpecial::F4 as u8, KeySpecial::F4 as u8),
    /*  63  */ KeyMappingEntry(KeySpecial::F5 as u8, KeySpecial::F5 as u8),
    /*  64  */ KeyMappingEntry(KeySpecial::F6 as u8, KeySpecial::F6 as u8),
    /*  65  */ KeyMappingEntry(KeySpecial::F7 as u8, KeySpecial::F7 as u8),
    /*  66  */ KeyMappingEntry(KeySpecial::F8 as u8, KeySpecial::F8 as u8),
    /*  67  */ KeyMappingEntry(KeySpecial::F9 as u8, KeySpecial::F9 as u8),
    /*  68  */ KeyMappingEntry(KeySpecial::F10 as u8, KeySpecial::F10 as u8),
    /*  69  */ KeyMappingEntry(KeySpecial::NumLock as u8, KeySpecial::NumLock as u8),
    /*  70  */ KeyMappingEntry(KeySpecial::ScrollLock as u8, KeySpecial::ScrollLock as u8),
    /*  71  */ KeyMappingEntry(KeySpecial::Home as u8, '7' as u8),
    /*  72  */ KeyMappingEntry(KeySpecial::Up as u8, '8' as u8),
    /*  73  */ KeyMappingEntry(KeySpecial::PageUp as u8, '9' as u8),
    /*  74  */ KeyMappingEntry('-' as u8, '-' as u8),
    /*  75  */ KeyMappingEntry(KeySpecial::Left as u8, '4' as u8),
    /*  76  */ KeyMappingEntry(KeySpecial::Center as u8, '5' as u8),
    /*  77  */ KeyMappingEntry(KeySpecial::Right as u8, '6' as u8),
    /*  78  */ KeyMappingEntry('+' as u8, '+' as u8),
    /*  79  */ KeyMappingEntry(KeySpecial::End as u8, '1' as u8),
    /*  80  */ KeyMappingEntry(KeySpecial::Down as u8, '2' as u8),
    /*  81  */ KeyMappingEntry(KeySpecial::PageDown as u8, '3' as u8),
    /*  82  */ KeyMappingEntry(KeySpecial::Insert as u8, '0' as u8),
    /*  83  */ KeyMappingEntry(KeySpecial::Delete as u8, '.' as u8),
    /*  84  */ KeyMappingEntry(KeySpecial::None as u8, KeySpecial::None as u8),
    /*  85  */ KeyMappingEntry(KeySpecial::None as u8, KeySpecial::None as u8),
    /*  86  */ KeyMappingEntry(KeySpecial::None as u8, KeySpecial::None as u8),
    /*  87  */ KeyMappingEntry(KeySpecial::F11 as u8, KeySpecial::F11 as u8),
    /*  88  */ KeyMappingEntry(KeySpecial::F12 as u8, KeySpecial::F12 as u8),
];

pub fn IsAlphabetScanCode(ScanCode: u8) -> bool {
    let resolved_code = KeyMappingTable[ScanCode as usize].NormalCode();
    if (('a' as u8) <= resolved_code) && (('z' as u8) >= resolved_code) {
        return true;
    }
    return false;
}

pub fn IsNumberOrSymbolScanCode(ScanCode: u8) -> bool {
    if (2 <= ScanCode) && (53 >= ScanCode) && !IsAlphabetScanCode(ScanCode) {
        return true;
    }
    return false;
}

pub fn IsNumberPadScancode(ScanCode: u8) -> bool {
    if (71 <= ScanCode) && (83 >= ScanCode) {
        return true;
    }
    return false;
}

pub fn IsUseCombinedCode(ScanCode: u8) -> bool {
    let DownScanCode = ScanCode & 0x7F;
    let mut UseCombinedKey = false;
    unsafe {
        if IsAlphabetScanCode(DownScanCode) {
            UseCombinedKey = if gs_stKeyboardManager.ShiftDown ^ gs_stKeyboardManager.CapsLockOn {
                true
            } else {
                false
            }
        } else if IsNumberOrSymbolScanCode(DownScanCode) {
            UseCombinedKey = if gs_stKeyboardManager.ShiftDown {
                true
            } else {
                false
            }
        } else if IsNumberPadScancode(DownScanCode) && !gs_stKeyboardManager.ExtendedCodeIn {
            UseCombinedKey = if gs_stKeyboardManager.NumLockOn {
                true
            } else {
                false
            }
        }
    }
    return UseCombinedKey;
}

pub fn UpdateCombinationKeyStatusAndLED(ScanCode: u8) {
    let (Down, DownScanCode) = if ScanCode & 0x80 != 0 {
        (false, ScanCode & 0x7F)
    } else {
        (true, ScanCode)
    };
    let mut LEDStatusChange = false;

    unsafe {
        if (DownScanCode == 42) || (DownScanCode == 54) {
            gs_stKeyboardManager.ShiftDown = Down;
        } else if (DownScanCode == 58) && Down {
            gs_stKeyboardManager.CapsLockOn ^= true;
            LEDStatusChange = true;
        } else if (DownScanCode == 69) && Down {
            gs_stKeyboardManager.NumLockOn ^= true;
            LEDStatusChange = true;
        } else if (DownScanCode == 70) && Down {
            gs_stKeyboardManager.ScrollLockOn ^= true;
            LEDStatusChange = true;
        }

        if LEDStatusChange {
            ChangeKeyboardLED(
                gs_stKeyboardManager.CapsLockOn,
                gs_stKeyboardManager.NumLockOn,
                gs_stKeyboardManager.ScrollLockOn,
            );
        }
    }
}

pub fn ConvertScanCodeToASCIICode(ScanCode: u8, ASCIICode: &mut u8, Flags: &mut u8) -> bool {
    unsafe {
        if gs_stKeyboardManager.SkipCountForPause > 0 {
            gs_stKeyboardManager.SkipCountForPause -= 1;
            return false;
        }
    }

    if ScanCode == 0xE1 {
        *ASCIICode = KeySpecial::Pause as u8;
        *Flags = KeyStatement::KeyFlagsDown as u8;
        unsafe {
            gs_stKeyboardManager.SkipCountForPause = KEY_SKIPCOUNTFORPAUSE;
        }
        return true;
    } else if ScanCode == 0xE0 {
        unsafe {
            gs_stKeyboardManager.ExtendedCodeIn = true;
        }
        return false;
    }

    if IsUseCombinedCode(ScanCode) {
        *ASCIICode = KeyMappingTable[(ScanCode & 0x7F) as usize].CombinedCode();
    } else {
        *ASCIICode = KeyMappingTable[(ScanCode & 0x7F) as usize].NormalCode();
    }

    unsafe {
        if gs_stKeyboardManager.ExtendedCodeIn {
            *Flags = KeyStatement::KeyFlagsExtendedkey as u8;
            gs_stKeyboardManager.ExtendedCodeIn = false;
        } else {
            *Flags = 0;
        }
    }

    if (ScanCode & 0x80) == 0 {
        *Flags |= KeyStatement::KeyFlagsDown as u8;
    }

    UpdateCombinationKeyStatusAndLED(ScanCode);
    return true;
}

pub fn InitializeKeyboard() -> bool {
    ActiveKeyboard()
}

pub fn ConvertScanCodeAndPutQueue(ScanCode: u8) -> bool {
    let mut key_data: KeyData = KeyData {
        ScanCode: ScanCode,
        ASCIICode: 0,
        Flags: 0,
    };
    let mut result = false;

    if ConvertScanCodeToASCIICode(ScanCode, &mut key_data.ASCIICode, &mut key_data.Flags) {
        let previous_interrupt = set_interrupt_flag(false);
        unsafe {
            KeyQueue.force_unlock();
            result = KeyQueue.lock().enqueue(key_data);
        }
        set_interrupt_flag(previous_interrupt);
    }
    result
}

pub fn GetKeyFromKeyQueue(data: &mut KeyData) -> bool {
    unsafe {
        if KeyQueue.lock().is_empty() {
            return true;
        }
        let previous_interrupt = set_interrupt_flag(false);
        let result = KeyQueue.lock().dequeue();
        set_interrupt_flag(previous_interrupt);
        match result {
            Ok(res) => {
                *data = res;
                return true;
            }
            Err(()) => {
                return false;
            }
        }
    }
}
