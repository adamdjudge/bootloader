use heapless::Vec;

use crate::memory::{E820Region, Type};
use crate::{mmu, println};

/// A loadable segment of a kernel executable.
pub struct Segment {
    pub addr: u32,
    pub size: usize,
    pub flags: u32,
    pub file_offset: usize,
    pub file_size: usize,
}

/// Contains kernel executable header info, including the start address and a list of segments.
pub struct KernelHeader {
    pub start_addr: u32,
    pub segments: Vec<Segment, 8>,
}

/// Contains the virtual and physical base address where the kernel executable was loaded.
pub struct KernelBase {
    pub virt_base: u32,
    pub phys_base: u32,
}

/// Trait representing a means of loading a kernel executable from a file on disk or some other
/// interface (e.g. serial, network) into memory.
pub trait Loader {
    type Error: Default;

    /// Loads the header of a kernel executable, and on success returns a `KernelHeader` struct
    /// containing information such as a list of segments and the execution start address. This
    /// needs to be done before loading the rest of the executable so that the segments can be
    /// mapped into virtual memory before data is read into them.
    fn load_header(&self) -> Result<KernelHeader, Self::Error>;

    /// Loads the segments of the kernel executable into memory, using a `KernelHeader` struct
    /// obtained from `load_header`.
    fn load_segments(&self, header: &KernelHeader) -> Result<(), Self::Error>;

    /// Loads a kernel executable into memory, and returns a `(KernelHeader, KernelBase)` tuple if
    /// successful. `mem_regions`, the list of detected physical memory regions, is used to
    /// determine the physical base address by finding a region large enough to fit the entire
    /// executable contiguously.
    fn load_kernel(
        &self,
        mem_regions: &[E820Region],
    ) -> Result<(KernelHeader, KernelBase), Self::Error> {
        let header = self.load_header()?;
        println!("start_addr=0x{:08x}", header.start_addr);
        for segment in &header.segments {
            println!(
                "segment: addr=0x{:08x} size=0x{:08x} flags=0x{:08x}",
                segment.addr, segment.size, segment.flags
            );
            println!(
                "         file_offset=0x{:08x} file_size=0x{:08x}",
                segment.file_offset, segment.file_size
            );
        }

        if header.start_addr < 0x80000000 {
            println!("error: start address is below 0x80000000");
            return Err(Self::Error::default());
        }
        if header.segments.is_empty() {
            println!("error: executable contains no segments");
            return Err(Self::Error::default());
        }

        let bottom_segment = header
            .segments
            .iter()
            .min_by(|x, y| x.addr.cmp(&y.addr))
            .unwrap();
        if bottom_segment.addr < 0x80000000 {
            println!("error: executable base address is below 0x80000000");
            return Err(Self::Error::default());
        }

        let top_segment = header
            .segments
            .iter()
            .max_by(|x, y| (x.addr + x.size as u32).cmp(&(y.addr + y.size as u32)))
            .unwrap();
        let total_span =
            (top_segment.addr + top_segment.size as u32 - bottom_segment.addr) as usize;
        let phys_base = match mem_regions
            .iter()
            .find(|r| r.rtype == Type::Free && r.addr >= 0x100000 && r.size >= total_span)
        {
            Some(region) => region.addr,
            None => {
                println!("error: not enough memory to load executable");
                return Err(Self::Error::default());
            }
        };

        for segment in &header.segments {
            mmu::map_range(phys_base, segment.addr, segment.size);
        }
        self.load_segments(&header)?;

        let kernel_addr = KernelBase {
            virt_base: bottom_segment.addr,
            phys_base: phys_base + (bottom_segment.addr & !mmu::PAGE_MASK),
        };
        Ok((header, kernel_addr))
    }
}

pub struct TestLoader;

impl Loader for TestLoader {
    type Error = ();

    fn load_header(&self) -> Result<KernelHeader, Self::Error> {
        let mut header = KernelHeader {
            start_addr: 0xC0000040,
            segments: Vec::new(),
        };

        let _ = header.segments.push(Segment {
            addr: 0xC0000000,
            size: 4096,
            flags: 0x1,
            file_offset: 0x1234,
            file_size: 0x5678,
        });
        let _ = header.segments.push(Segment {
            addr: 0xC0001000,
            size: 4096,
            flags: 0x2,
            file_offset: 0x5678,
            file_size: 0xabcd,
        });

        Ok(header)
    }

    fn load_segments(&self, header: &KernelHeader) -> Result<(), Self::Error> {
        let _ = header;
        Ok(())
    }
}
