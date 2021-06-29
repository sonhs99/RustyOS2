use crate::assembly::OutPortByte;

const PIC_MASTER_PORT1: u16 = 0x20;
const PIC_MASTER_PORT2: u16 = 0x21;
const PIC_SLAVE_PORT1:  u16 = 0xA0;
const PIC_SLAVE_PORT2:  u16 = 0xA1;

pub const PIC_IRQSTARTVECTOR: u8 = 0x20;

pub fn InitializePIC() {
    OutPortByte(PIC_MASTER_PORT1, 0x11);
    OutPortByte(PIC_MASTER_PORT2, PIC_IRQSTARTVECTOR);
    OutPortByte(PIC_MASTER_PORT2, 0x04);
    OutPortByte(PIC_MASTER_PORT2, 0x01);
    OutPortByte(PIC_SLAVE_PORT1, 0x11);
    OutPortByte(PIC_SLAVE_PORT1, PIC_IRQSTARTVECTOR + 8);
    OutPortByte(PIC_SLAVE_PORT1, 0x02);
    OutPortByte(PIC_SLAVE_PORT1, 0x01);
}

pub fn MaskedPICInterrupt( IRQBitmask: u16 ) {
    OutPortByte(PIC_MASTER_PORT2, (IRQBitmask & 0xFFFF) as u8);
    OutPortByte(PIC_SLAVE_PORT2, (IRQBitmask >> 8) as u8);
}

pub fn SendEOI(IRQNumber: u16) {
    OutPortByte(PIC_MASTER_PORT1, 0x20);
    if IRQNumber >= 8 {
        OutPortByte(PIC_SLAVE_PORT1, 0x20);
    }

}