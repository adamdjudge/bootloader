use core::arch::asm;
use core::ops::{Index, IndexMut};

// Defined in linker script.
unsafe extern "C" {
    static __bootloader_base: u8;
    static __bootloader_data: u8;
    static __bootloader_top: u8;
    static __loram_top: u8;
}

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_MASK: u32 = 0xfffff000;

static mut NEXT_LOMEM_FRAME: u32 = 0;

#[inline]
pub const fn page_align_up(addr: u32) -> u32 {
    (addr + PAGE_SIZE as u32 - 1) & PAGE_MASK
}

#[inline]
pub const fn page_align_down(addr: u32) -> u32 {
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

    const fn is_present(&self) -> bool {
        self.0 & Self::PRESENT != 0
    }

    fn set_writable(&mut self, value: bool) {
        if value {
            self.0 |= Self::WRITABLE;
        } else {
            self.0 &= !Self::WRITABLE;
        }
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

    fn alloc() -> &'static mut Self {
        let stack_base = &raw const __loram_top as u32 - 65 * PAGE_SIZE as u32;
        unsafe {
            assert!(NEXT_LOMEM_FRAME < stack_base);
            let frame = NEXT_LOMEM_FRAME;
            NEXT_LOMEM_FRAME += PAGE_SIZE as u32;
            &mut *(frame as *mut _)
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

/// Initializes virtual memory management and enables paging. This functions must be called prior to
/// calling any other functions in this module.
pub fn init() {
    // Initialize the page table frame allocator. Frames to use as page tables are taken from low
    // RAM above the bootloader executable image.
    unsafe {
        NEXT_LOMEM_FRAME = page_align_up(&raw const __bootloader_top as u32);
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
        PAGE_DIRECTORY[0] = PageTableEntry::new(page_table as *const _ as u32).with_writable();
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

/// Maps a contiguous range of the virtual address space, with size given in bytes, to a contiguous
/// range of physical memory starting at `phys_base`, which is expected to be the page-aligned base
/// address of a physical memory region. All virtual addresses within the range will thus have the
/// same offset from the physical addresses they are mapped to. The entire virtual address range is
/// mapped as both readable and writable.
pub fn map_range(phys_base: u32, vaddr: u32, size: usize) {
    let vbase = page_align_down(vaddr);
    let vtop = vaddr + size as u32;
    let map_offset = vbase - phys_base;

    for vaddr in (vbase..vtop).step_by(PAGE_SIZE) {
        // Allocate a new page table if one is not already present for this virtual address.
        let pte_slot = unsafe { &mut PAGE_DIRECTORY[(vaddr >> 22) as usize] };
        if !pte_slot.is_present() {
            let pt = PageTable::alloc();
            let pte = PageTableEntry::new(pt as *const _ as u32).with_writable();
            *pte_slot = pte;
        }

        // Map a physical frame if one is not already present for this virtual address.
        let ptes_base = 0x7FC00000 as *mut PageTableEntry;
        let pte_slot = unsafe { &mut *ptes_base.add((vaddr >> 12) as usize) };
        if !pte_slot.is_present() {
            *pte_slot = PageTableEntry::new(vaddr - map_offset).with_writable();
        }
    }
}

/// Marks a range of the virtual address space, with size given in bytes, as read-only. Unmapped
/// portions within the range are skipped.
pub fn write_protect_range(vaddr: u32, size: usize) {
    let vbase = page_align_down(vaddr);
    let vtop = vaddr + size as u32;

    for vaddr in (vbase..vtop).step_by(PAGE_SIZE) {
        let pte_slot = unsafe { &PAGE_DIRECTORY[(vaddr >> 22) as usize] };
        if pte_slot.is_present() {
            let ptes_base = 0x7FC00000 as *mut PageTableEntry;
            let pte_slot = unsafe { &mut *ptes_base.add((vaddr >> 12) as usize) };
            pte_slot.set_writable(false);
        }
    }

    // Reload page directory to flush the TLB.
    unsafe {
        asm!("mov eax, cr3; mov cr3, eax");
    }
}
