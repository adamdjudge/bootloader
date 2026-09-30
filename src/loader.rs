use core::cmp::Ordering;

use heapless::Vec;

use crate::{mmu, println};

/// A loadable segment of a kernel executable.
#[derive(PartialEq, Eq)]
pub struct Segment {
    pub addr: u32,
    pub size: usize,
    pub flags: u32,
    pub file_offset: usize,
    pub file_size: usize,
}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> Ordering {
        self.addr.cmp(&other.addr)
    }
}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Contains kernel executable header info, including the start address and a list of segments.
pub struct KernelHeader {
    pub start_addr: u32,
    pub segments: Vec<Segment, 8>,
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

    /// Loads a kernel executable into memory, and returns a `KernelHeader` if successful.
    fn load_kernel(&self) -> Result<KernelHeader, Self::Error> {
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

        let virt_base = match header.segments.iter().min() {
            Some(segment) => segment.addr,
            None => {
                println!("error: executable contains no segments"); 
                return Err(Self::Error::default());
            }
        };
        if virt_base < 0x80000000 {
            println!("error: executable base address is below 0x80000000");
            return Err(Self::Error::default());
        }

        for segment in &header.segments {
            mmu::map_range(segment.addr, segment.size);
        }
        self.load_segments(&header)?;

        Ok(header)
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
