use core::arch::asm;

/// Write an 8-bit value to an I/O port.
pub unsafe fn out8(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") val);
    }
}

/// Read an 8-bit value from an I/O port.
pub unsafe fn in8(port: u16) -> u8 {
    let val: u8;
    unsafe {
        asm!("in al, dx", in("dx") port, lateout("al") val);
    }
    val
}
