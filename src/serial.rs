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

/// Serial port designation, either COM1 or COM2.
#[allow(dead_code)]
#[derive(Debug)]
pub enum ComPort {
    Com1,
    Com2,
}

impl ComPort {
    /// Returns I/O base address of the COM port.
    fn base_addr(&self) -> u16 {
        match self {
            ComPort::Com1 => 0x3f8,
            ComPort::Com2 => 0x2f8,
        }
    }
}

/// Handle for a serial port.
pub struct SerialPort {
    base: u16,
}

impl SerialPort {
    /// Returns a handle to the specified serial port, configured to a given baud rate. Panics if
    /// the baud rate is invalid.
    pub fn get(com: ComPort, baudrate: usize) -> Self {
        let serial = Self {
            base: com.base_addr(),
        };
        serial.set_baud_rate(baudrate);
        serial.reset();
        serial
    }

    /// Resets the serial port.
    pub fn reset(&self) {
        unsafe {
            port::out8(self.base + IEN, 0x00); // Disable interrupts
            port::out8(self.base + LCR, 0x03); // 8 bits, no parity, 1 stop bit
            port::out8(self.base + FCR, 0x07); // Reset buffers and enable FIFOs
        }
    }

    /// Configures the baud rate of the serial port. Panics if the given baud rate is invalid.
    pub fn set_baud_rate(&self, rate: usize) {
        assert!(
            rate >= MIN_BAUD && rate <= MAX_BAUD && MAX_BAUD % rate == 0,
            "tried to set invalid baud rate {rate}"
        );

        let divisor = MAX_BAUD / rate;
        unsafe {
            let lcr = port::in8(self.base + LCR);
            port::out8(self.base + LCR, lcr | 0x80); // Set DLAB
            port::out8(self.base + DIV_LO, (divisor & 0xff) as u8);
            port::out8(self.base + DIV_HI, (divisor >> 8) as u8);
            port::out8(self.base + LCR, lcr);
        }
    }

    /// Receives one byte from the serial port. Blocks execution until there is a byte available.
    pub fn receive_u8(&self) -> u8 {
        unsafe {
            while port::in8(self.base + LSR) & 0x01 == 0 {}
            port::in8(self.base + FIFO)
        }
    }

    /// Receives one 32-bit value from the serial port, transmitted in little endian order. Blocks
    /// execution until 4 bytes have been received.
    pub fn receive_u32(&self) -> u32 {
        let mut bytes = [0u8; 4];
        bytes.fill_with(|| self.receive_u8());
        u32::from_le_bytes(bytes)
    }
}

#[derive(Default, Clone, Copy)]
struct Segment {
    addr: u32,
    size: u32,
}

/// Loads a kernel into memory over the given serial port, and returns its start address. Blocks
/// execution until a full executable has been loaded. Panics if the received executable is invalid
/// or if there is a checksum failure.
/// 
/// The kernel executable image received over serial is expected to contain a header, made of little
/// endian u32 fields, followed by a byte stream of all the segments concatenated together. The
/// header has the following format:
///   - Start address (u32)
///   - Number of segments (u32)
///   - Array with entry for each segment:
///     - Base address (u32)
///     - Size in bytes (u32)
///   - CRC-32 data checksum (u32)
///   - CRC-32 header checksum (u32)
/// 
/// NOTE: Use sendelf.py to parse ELF executables and emit an image to send over serial.
pub fn load_kernel(serial: &SerialPort) -> u32 {
    let writer = Writer::get();
    let mut crc = Crc32::new();

    // Receive start address dword.
    let start_addr = serial.receive_u32();
    crc.crc32_u32(start_addr);
    let _ = write!(writer, "start address: 0x{:08x}\n", start_addr);

    // Receive segments count dword.
    let segments_count = serial.receive_u32() as usize;
    crc.crc32_u32(segments_count as u32);
    assert!(
        segments_count <= MAX_SEGMENTS,
        "segment count {} is too large, max supported is {}",
        segments_count,
        MAX_SEGMENTS,
    );
    let _ = write!(writer, "segments count: {}\n", segments_count);

    // Receive segments array. Each segment contains an address dword and size dword.
    let mut segments = [Segment::default(); MAX_SEGMENTS];
    for i in 0..segments_count {
        let segment = Segment {
            addr: serial.receive_u32(),
            size: serial.receive_u32(),
        };
        crc.crc32_u32(segment.addr);
        crc.crc32_u32(segment.size);
        segments[i] = segment;
    }

    // Receive data checksum dword.
    let data_checksum = serial.receive_u32();
    crc.crc32_u32(data_checksum);

    // Receive header checksum dword and check for correctness.
    let header_checksum = serial.receive_u32();
    let crc32 = crc.finish();
    assert!(
        header_checksum == crc32,
        "header checksum 0x{:x} does not match computed CRC-32 0x{:x}",
        header_checksum,
        crc32
    );

    // Receive data for each segment and load into memory, computing data checksum from all bytes.
    let mut crc = Crc32::new();
    for segment in segments.iter().take(segments_count) {
        let _ = write!(
            writer,
            "load segment: addr=0x{:08x} size=0x{:08x}\n",
            segment.addr, segment.size
        );

        let segment_slice =
            unsafe { slice::from_raw_parts_mut(segment.addr as *mut u8, segment.size as usize) };
        segment_slice.fill_with(|| serial.receive_u8());
        crc.crc32_slice(segment_slice);
    }

    // Check data checksum for correctness.
    let crc32 = crc.finish();
    assert!(
        data_checksum == crc32,
        "data checksum 0x{:x} does not match computed CRC-32 0x{:x}",
        data_checksum,
        crc32
    );

    // Return the kernel executable start address so the bootloader can jump to it.
    start_addr
}
