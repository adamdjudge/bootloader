use crate::port;

/// (DLAB=0) (RW) Data FIFO offset
const FIFO: u16 = 0;
/// (DLAB=0) (RW) Interrupt enable register offset
const IEN: u16 = 1;
/// (DLAB=1) (RW) Divisor high byte offset
const DIV_LO: u16 = 0;
/// (DLAB=1) (RW) Divisor low byte offset
const DIV_HI: u16 = 1;
/// (WO) FIFO control register offset
const FCR: u16 = 2;
/// (RW) Line control register offset
const LCR: u16 = 3;
/// (RO) Line status register offset
const LSR: u16 = 5;

/// Minimum baud rate
const MIN_BAUD: usize = 2;
const MAX_BAUD: usize = 115200;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ComPort {
    Com1,
    Com2,
}

impl ComPort {
    fn base_addr(&self) -> u16 {
        match self {
            ComPort::Com1 => 0x3f8,
            ComPort::Com2 => 0x2f8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BaudRate(usize);

impl TryFrom<usize> for BaudRate {
    type Error = usize;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value >= MIN_BAUD && value <= MAX_BAUD && MAX_BAUD % value == 0 {
            Ok(BaudRate(value))
        } else {
            Err(value)
        }
    }
}

pub struct SerialPort {
    base: u16,
}

impl SerialPort {
    pub fn get(com: ComPort) -> Self {
        Self {
            base: com.base_addr(),
        }
    }

    pub fn reset(&self) {
        unsafe {
            port::out8(self.base + IEN, 0x00); // Disable interrupts
            port::out8(self.base + LCR, 0x03); // 8 bits, no parity, 1 stop bit
            port::out8(self.base + FCR, 0x07); // Reset buffers and enable FIFOs
        }
    }

    pub fn set_baud_rate(&self, baud: BaudRate) {
        let divisor = MAX_BAUD / baud.0;
        unsafe {
            let lcr = port::in8(self.base + LCR);
            port::out8(self.base + LCR, lcr | 0x80); // Set DLAB
            port::out8(self.base + DIV_LO, (divisor & 0xff) as u8);
            port::out8(self.base + DIV_HI, (divisor >> 8) as u8);
            port::out8(self.base + LCR, lcr);
        }
    }

    pub fn receive_byte(&self) -> u8 {
        unsafe {
            while port::in8(self.base + LSR) & 0x01 == 0 {}
            port::in8(self.base + FIFO)
        }
    }

    pub fn receive_dword(&self) -> u32 {
        let mut bytes = [0u8; 4];
        for i in 0..4 {
            bytes[i] = self.receive_byte();
        }
        u32::from_be_bytes(bytes)
    }
}
