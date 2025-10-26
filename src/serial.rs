use core::fmt::Write;
use core::slice;

use crate::console::Writer;
use crate::crc32::Crc32;
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
/// Maximum baud rate
const MAX_BAUD: usize = 115200;

/// Maximum number of program segments that can be loaded over serial
const MAX_SEGMENTS: usize = 16;

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

    pub fn receive_u32(&self) -> u32 {
        let mut bytes = [0u8; 4];
        for i in 0..4 {
            bytes[i] = self.receive_byte();
        }
        u32::from_le_bytes(bytes)
    }
}

#[derive(Default, Clone, Copy)]
struct Segment {
    addr: u32,
    size: u32,
}

pub fn load_kernel(serial: &SerialPort) -> u32 {
    let mut segments = [Segment::default(); MAX_SEGMENTS];
    let mut writer = Writer::get();
    let mut crc = Crc32::new();

    let start_addr = serial.receive_u32();
    crc.crc32_u32(start_addr);
    let _ = write!(&mut writer, "start address: 0x{:08x}\n", start_addr);

    let segments_count = serial.receive_u32() as usize;
    crc.crc32_u32(segments_count as u32);
    assert!(
        segments_count <= MAX_SEGMENTS,
        "segment count {} is too large, max supported is {}",
        segments_count,
        MAX_SEGMENTS,
    );
    let _ = write!(&mut writer, "segments count: {}\n", segments_count);

    for i in 0..segments_count {
        let segment = Segment {
            addr: serial.receive_u32(),
            size: serial.receive_u32(),
        };
        crc.crc32_u32(segment.addr);
        crc.crc32_u32(segment.size);
        segments[i] = segment;
    }

    let data_checksum = serial.receive_u32();
    crc.crc32_u32(data_checksum);

    let header_checksum = serial.receive_u32();
    let crc32 = crc.finish();
    assert_eq!(
        header_checksum, crc32,
        "header checksum 0x{:x} does not match computed CRC-32 0x{:x}",
        header_checksum, crc32
    );

    for segment in segments.iter().take(segments_count) {
        let _ = write!(
            &mut writer,
            "load segment: addr=0x{:08x} size=0x{:08x}\n",
            segment.addr, segment.size
        );
        let segment =
            unsafe { slice::from_raw_parts_mut(segment.addr as *mut u8, segment.size as usize) };
        for i in 0..segment.len() {
            segment[i] = serial.receive_byte();
        }
    }

    let mut crc = Crc32::new();
    for segment in segments.iter().take(segments_count) {
        crc.crc32_slice(unsafe {
            slice::from_raw_parts_mut(segment.addr as *mut u8, segment.size as usize)
        });
    }
    let crc32 = crc.finish();
    assert_eq!(
        data_checksum, crc32,
        "data checksum 0x{:x} does not match computed CRC-32 0x{:x}",
        data_checksum, crc32
    );

    start_addr
}
