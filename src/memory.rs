use core::slice;

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Type {
    None,
    Free,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
}

/// Memory region descriptor returned by the BIOS 0xE820 call.
#[repr(C)]
pub struct E820Region {
    pub addr: u32,
    pad1: u32,
    pub size: u32,
    pad2: u32,
    pub rtype: Type,
    pub acpi_attr: u32,
}

/// Returns a slice of memory regions detected by the BIOS.
pub fn get_regions() -> &'static [E820Region] {
    // Defined in start.s
    unsafe extern "C" {
        static e820_regions: E820Region;
        static e820_regions_count: usize;
    }

    unsafe { slice::from_raw_parts(&raw const e820_regions, e820_regions_count) }
}
