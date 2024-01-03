use crate::assembly::{InPortByte, OutPortByte};

const PIT_FREQUENCY: u64 = 1193180;

const PIT_PORT_CONTROL: u16 = 0x43;
const PIT_PORT_COUNTER0: u16 = 0x40;

const PIT_CONTROL_COUNTER0: u8 = 0x00;
const PIT_CONTROL_LSBMSBRW: u8 = 0x30;
const PIT_CONTROL_LATCH: u8 = 0x00;
const PIT_CONTROL_MODE0: u8 = 0x00;
const PIT_CONTROL_MODE2: u8 = 0x04;
const PIT_CONTROL_BINARYCOUNTER: u8 = 0x00;
const PIT_CONTROL_BCDCOUNTER: u8 = 0x01;

const PIT_COUNTER0_ONCE: u8 =
    PIT_CONTROL_COUNTER0 | PIT_CONTROL_LSBMSBRW | PIT_CONTROL_MODE0 | PIT_CONTROL_BINARYCOUNTER;
const PIT_COUNTER0_PERIODIC: u8 =
    PIT_CONTROL_COUNTER0 | PIT_CONTROL_LSBMSBRW | PIT_CONTROL_MODE2 | PIT_CONTROL_BINARYCOUNTER;

pub fn convert_from_ms(time: u64) -> u64 {
    PIT_FREQUENCY * time / 1000
}

pub fn convert_from_us(time: u64) -> u64 {
    PIT_FREQUENCY * time / 1000000
}

pub fn init_PIT(count: u16, periodic: bool) {
    OutPortByte(PIT_PORT_CONTROL, PIT_COUNTER0_ONCE);
    if periodic {
        OutPortByte(PIT_PORT_CONTROL, PIT_COUNTER0_PERIODIC);
    }
    OutPortByte(PIT_PORT_COUNTER0, (count & 0xFF) as u8);
    OutPortByte(PIT_PORT_COUNTER0, (count >> 8) as u8);
}

pub fn read_counter0() -> u16 {
    OutPortByte(PIT_PORT_COUNTER0, PIT_CONTROL_LATCH);
    let low_byte = InPortByte(PIT_PORT_COUNTER0) as u16;
    let high_byte = InPortByte(PIT_PORT_COUNTER0) as u16;
    low_byte | (high_byte << 8)
}

pub fn wait_using_PIT(count: u16) {
    init_PIT(count, true);
    let last_count = read_counter0();
    loop {
        let current_count = read_counter0();
        if (last_count - current_count) & 0xFFFF >= count {
            break;
        }
    }
}

pub fn wait(milisecond: u64) {
    for _ in 0..milisecond / 30 {
        wait_using_PIT(convert_from_ms(30) as u16);
    }
    wait_using_PIT(convert_from_ms(milisecond % 30) as u16);
}

const RTC_CMOSADDRESS: u16 = 0x70;
const RTC_CMOSDATA: u16 = 0x71;

const RTC_ADDRESS_SECOND: u8 = 0x00;
const RTC_ADDRESS_MINUTE: u8 = 0x02;
const RTC_ADDRESS_HOUR: u8 = 0x04;
const RTC_ADDRESS_DAYOFWEEK: u8 = 0x06;
const RTC_ADDRESS_DAYOFMONTH: u8 = 0x07;
const RTC_ADDRESS_MONTH: u8 = 0x08;
const RTC_ADDRESS_YEAR: u8 = 0x09;

#[repr(C, packed(1))]
pub struct Date {
    pub year: u8,
    pub month: u8,
    pub day_of_month: u8,
    pub day_of_week: u8,
}

pub struct Time {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

fn convert_bcd_to_bin(bcd: u8) -> u8 {
    (bcd >> 4) * 10 + (bcd & 0xF)
}

impl Date {
    pub fn current() -> Self {
        OutPortByte(RTC_CMOSADDRESS, RTC_ADDRESS_YEAR);
        let data = InPortByte(RTC_CMOSDATA);
        let year = convert_bcd_to_bin(data);

        OutPortByte(RTC_CMOSADDRESS, RTC_ADDRESS_MONTH);
        let data = InPortByte(RTC_CMOSDATA);
        let month = convert_bcd_to_bin(data);

        OutPortByte(RTC_CMOSADDRESS, RTC_ADDRESS_DAYOFMONTH);
        let data = InPortByte(RTC_CMOSDATA);
        let day_of_month = convert_bcd_to_bin(data);

        OutPortByte(RTC_CMOSADDRESS, RTC_ADDRESS_DAYOFWEEK);
        let data = InPortByte(RTC_CMOSDATA);
        let day_of_week = convert_bcd_to_bin(data);

        Self {
            year,
            month,
            day_of_month,
            day_of_week,
        }
    }

    pub fn week_string(&self) -> &'static str {
        static WEEK_STRING: [&'static str; 8] = [
            "Error",
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ];

        if self.day_of_week >= 8 {
            WEEK_STRING[0]
        } else {
            WEEK_STRING[self.day_of_week as usize]
        }
    }
}

impl Time {
    pub fn current() -> Self {
        OutPortByte(RTC_CMOSADDRESS, RTC_ADDRESS_HOUR);
        let data = InPortByte(RTC_CMOSDATA);
        let hour = convert_bcd_to_bin(data);

        OutPortByte(RTC_CMOSADDRESS, RTC_ADDRESS_MINUTE);
        let data = InPortByte(RTC_CMOSDATA);
        let minute = convert_bcd_to_bin(data);

        OutPortByte(RTC_CMOSADDRESS, RTC_ADDRESS_SECOND);
        let data = InPortByte(RTC_CMOSDATA);
        let second = convert_bcd_to_bin(data);

        Self {
            hour,
            minute,
            second,
        }
    }
}
