use core::arch::asm;
use core::ops::{Index, IndexMut};

// Defined in linker script.
unsafe extern "C" {
    static __bootloader_base: u8;
    static __bootloader_data: u8;
    static __bootloader_top: u8;
    static __loram_top: u8;
}

const PAGE_SIZE: usize = 4096;
const PAGE_MASK: u32 = 0xfffff000;

static mut NEXT_FRAME: u32 = 0;

#[inline]
const fn page_align_up(addr: u32) -> u32 {
    (addr + PAGE_SIZE as u32 - 1) & PAGE_MASK
}

#[inline]
const fn page_align_down(addr: u32) -> u32 {
    addr & PAGE_MASK
}

#[repr(transparent)]
struct PageTableEntry(u32);

impl PageTableEntry {
    const PRESENT: u32 = 0x1;
    const WRITABLE: u32 = 0x2;

    const fn empty() -> Self {
        Self(0)
    }

    const fn new(paddr: u32) -> Self {
        Self((paddr & PAGE_MASK) | Self::PRESENT)
    }

    const fn with_writable(&self) -> Self {
        Self(self.0 | Self::WRITABLE)
    }
}

#[repr(align(4096))]
struct PageTable {
    entries: [PageTableEntry; 1024],
}

impl PageTable {
    const fn new() -> Self {
        Self {
            entries: [const { PageTableEntry::empty() }; 1024],
        }
    }

    const fn alloc() -> &'static mut Self {
        unsafe {
            let frame = NEXT_FRAME;
            NEXT_FRAME += PAGE_SIZE as u32;
            &mut *(frame as *mut PageTable)
        }
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

static mut PAGE_DIRECTORY: PageTable = PageTable::new();

pub fn init() {
    // Initialize the page table frame allocator. Frames to use as page tables are taken from low
    // RAM above the bootloader executable image.
    unsafe {
        NEXT_FRAME = page_align_up(&raw const __bootloader_top as u32);
    }

    // Allocate an initial page table, and map VGA text memory.
    let page_table = PageTable::alloc();
    page_table[0xb8] = PageTableEntry::new(0xb8000).with_writable();

    // Map the bootloader executable image.
    let mut addr = &raw const __bootloader_base as u32;
    while addr < &raw const __bootloader_top as u32 {
        if addr < &raw const __bootloader_data as u32 {
            page_table[(addr >> 12) as usize] = PageTableEntry::new(addr);
        } else {
            page_table[(addr >> 12) as usize] = PageTableEntry::new(addr).with_writable();
        }
        addr += PAGE_SIZE as u32;
    }

    // Map 64 KiB of stack at the top of low RAM.
    addr = &raw const __loram_top as u32;
    for _ in 0..16 {
        addr -= PAGE_SIZE as u32;
        page_table[(addr >> 12) as usize] = PageTableEntry::new(addr).with_writable();
    }

    unsafe {
        // Set up the page directory. The initial page table is linked to create a complete mapping.
        // Entry 511 is linked to the page directory itself, so that 0x7FC00000 - 0x7FFFFFFF in the
        // virtual address space will map to an array of all mapped page tables.
        PAGE_DIRECTORY[0] = PageTableEntry::new(page_table as *const PageTable as u32).with_writable();
        PAGE_DIRECTORY[511] = PageTableEntry::new(&raw const PAGE_DIRECTORY as u32).with_writable();

        // Load the page directory and enable paging.
        asm!(
            "mov cr3, eax
            mov eax, cr0
            or eax, 0x80000000
            mov cr0, eax",
            in("eax") &raw const PAGE_DIRECTORY
        );
    }
}
