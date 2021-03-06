/// Program header table entry unused
pub const PT_NULL: u32 = 0;
/// Loadable program segment
pub const PT_LOAD: u32 = 1;
/// Dynamic linking information
pub const PT_DYNAMIC: u32 = 2;
/// Program interpreter
pub const PT_INTERP: u32 = 3;
/// Auxiliary information
pub const PT_NOTE: u32 = 4;
/// Reserved
pub const PT_SHLIB: u32 = 5;
/// Entry for header table itself
pub const PT_PHDR: u32 = 6;
/// Thread-local storage segment
pub const PT_TLS: u32 = 7;
/// Number of defined types
pub const PT_NUM: u32 = 8;
/// Start of OS-specific
pub const PT_LOOS: u32 = 0x60000000;
/// GCC .eh_frame_hdr segment
pub const PT_GNU_EH_FRAME: u32 = 0x6474e550;
/// Indicates stack executability
pub const PT_GNU_STACK: u32 = 0x6474e551;
/// Read-only after relocation
pub const PT_GNU_RELRO: u32 = 0x6474e552;
/// Sun Specific segment
pub const PT_LOSUNW: u32 = 0x6ffffffa;
/// Sun Specific segment
pub const PT_SUNWBSS: u32 = 0x6ffffffa;
/// Stack segment
pub const PT_SUNWSTACK: u32 = 0x6ffffffb;
/// End of OS-specific
pub const PT_HISUNW: u32 = 0x6fffffff;
/// End of OS-specific
pub const PT_HIOS: u32 = 0x6fffffff;
/// Start of processor-specific
pub const PT_LOPROC: u32 = 0x70000000;
/// ARM unwind segment
pub const PT_ARM_EXIDX: u32 = 0x70000001;
/// End of processor-specific
pub const PT_HIPROC: u32 = 0x7fffffff;

/// Segment is executable
pub const PF_X: u32 = 1 << 0;

/// Segment is writable
pub const PF_W: u32 = 1 << 1;

/// Segment is readable
pub const PF_R: u32 = 1 << 2;

pub fn pt_to_str(pt: u32) -> &'static str {
    match pt {
        PT_NULL => "PT_NULL",
        PT_LOAD => "PT_LOAD",
        PT_DYNAMIC => "PT_DYNAMIC",
        PT_INTERP => "PT_INTERP",
        PT_NOTE => "PT_NOTE",
        PT_SHLIB => "PT_SHLIB",
        PT_PHDR => "PT_PHDR",
        PT_TLS => "PT_TLS",
        PT_NUM => "PT_NUM",
        PT_LOOS => "PT_LOOS",
        PT_GNU_EH_FRAME => "PT_GNU_EH_FRAME",
        PT_GNU_STACK => "PT_GNU_STACK",
        PT_GNU_RELRO => "PT_GNU_RELRO",
        PT_SUNWBSS => "PT_SUNWBSS",
        PT_SUNWSTACK => "PT_SUNWSTACK",
        PT_HIOS => "PT_HIOS",
        PT_LOPROC => "PT_LOPROC",
        PT_HIPROC => "PT_HIPROC",
        PT_ARM_EXIDX => "PT_ARM_EXIDX",
        _ => "UNKNOWN_PT",
    }
}

if_std! {
    use core::fmt;
    use scroll::ctx;
    use core::result;
    use core::ops::Range;
    use container::{Ctx, Container};

    #[derive(Default, PartialEq, Clone)]
    /// A unified ProgramHeader - convertable to and from 32-bit and 64-bit variants
    pub struct ProgramHeader {
        pub p_type  : u32,
        pub p_flags : u32,
        pub p_offset: u64,
        pub p_vaddr : u64,
        pub p_paddr : u64,
        pub p_filesz: u64,
        pub p_memsz : u64,
        pub p_align : u64,
    }

    impl ProgramHeader {
        /// Return the size of the underlying program header, given a `Ctx`
        #[inline]
        pub fn size(ctx: &Ctx) -> usize {
            use scroll::ctx::SizeWith;
            Self::size_with(ctx)
        }
        /// Create a new `PT_LOAD` ELF program header
        pub fn new() -> Self {
            ProgramHeader {
                p_type  : PT_LOAD,
                p_flags : 0,
                p_offset: 0,
                p_vaddr : 0,
                p_paddr : 0,
                p_filesz: 0,
                p_memsz : 0,
                //TODO: check if this is true for 32-bit pt_load
                p_align : 2 << 20,
            }
        }
        pub fn to_range(&self) -> Range<usize> {
            (self.p_offset as usize..self.p_offset as usize + self.p_filesz as usize)
        }
        /// Sets the executable flag
        pub fn executable(&mut self) {
            self.p_flags |= PF_X;
        }
        /// Sets the write flag
        pub fn write(&mut self) {
            self.p_flags |= PF_W;
        }
        /// Sets the read flag
        pub fn read(&mut self) {
            self.p_flags |= PF_R;
        }

        #[cfg(feature = "endian_fd")]
        pub fn parse(bytes: &[u8], mut offset: usize, count: usize, ctx: Ctx) -> ::error::Result<Vec<ProgramHeader>> {
            use scroll::Pread;
            let mut program_headers = Vec::with_capacity(count);
            for _ in 0..count {
                let phdr = bytes.gread_with(&mut offset, ctx)?;
                program_headers.push(phdr);
            }
            Ok(program_headers)
        }
    }

    impl fmt::Debug for ProgramHeader {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f,
                   "p_type: {} p_flags 0x{:x} p_offset: 0x{:x} p_vaddr: 0x{:x} p_paddr: 0x{:x} \
                    p_filesz: 0x{:x} p_memsz: 0x{:x} p_align: {}",
                   pt_to_str(self.p_type),
                   self.p_flags,
                   self.p_offset,
                   self.p_vaddr,
                   self.p_paddr,
                   self.p_filesz,
                   self.p_memsz,
                   self.p_align)
        }
    }

    impl ctx::SizeWith<Ctx> for ProgramHeader {
        type Units = usize;
        fn size_with(ctx: &Ctx) -> usize {
            match ctx.container {
                Container::Little => {
                    program_header32::SIZEOF_PHDR
                },
                Container::Big => {
                    program_header64::SIZEOF_PHDR
                },
            }
        }
    }

    impl<'a> ctx::TryFromCtx<'a, Ctx> for ProgramHeader {
        type Error = ::error::Error;
        type Size = usize;
        fn try_from_ctx(bytes: &'a [u8], Ctx { container, le}: Ctx) -> result::Result<(Self, Self::Size), Self::Error> {
            use scroll::Pread;
            let res = match container {
                Container::Little => {
                    (bytes.pread_with::<program_header32::ProgramHeader>(0, le)?.into(), program_header32::SIZEOF_PHDR)
                },
                Container::Big => {
                    (bytes.pread_with::<program_header64::ProgramHeader>(0, le)?.into(), program_header64::SIZEOF_PHDR)
                }
            };
            Ok(res)
        }
    }

    impl ctx::TryIntoCtx<Ctx> for ProgramHeader {
        type Error = ::error::Error;
        type Size = usize;
        fn try_into_ctx(self, bytes: &mut [u8], Ctx {container, le}: Ctx) -> result::Result<Self::Size, Self::Error> {
            use scroll::Pwrite;
            match container {
                Container::Little => {
                    let phdr: program_header32::ProgramHeader = self.into();
                    Ok(bytes.pwrite_with(phdr, 0, le)?)
                },
                Container::Big => {
                    let phdr: program_header64::ProgramHeader = self.into();
                    Ok(bytes.pwrite_with(phdr, 0, le)?)
                }
            }
        }
    }
}

macro_rules! elf_program_header_std_impl { ($size:ty) => {

    #[cfg(test)]
    mod test {
        use super::*;
        #[test]
        fn size_of() {
            assert_eq!(::std::mem::size_of::<ProgramHeader>(), SIZEOF_PHDR);
        }
    }

    if_std! {

        use elf::program_header::ProgramHeader as ElfProgramHeader;
        use error::Result;

        use core::slice;
        use core::fmt;

        use std::fs::File;
        use std::io::{Seek, Read};
        use std::io::SeekFrom::Start;

        use plain::Plain;

        impl From<ProgramHeader> for ElfProgramHeader {
            fn from(ph: ProgramHeader) -> Self {
                ElfProgramHeader {
                    p_type   : ph.p_type,
                    p_flags  : ph.p_flags,
                    p_offset : ph.p_offset as u64,
                    p_vaddr  : ph.p_vaddr as u64,
                    p_paddr  : ph.p_paddr as u64,
                    p_filesz : ph.p_filesz as u64,
                    p_memsz  : ph.p_memsz as u64,
                    p_align  : ph.p_align as u64,
                }
            }
        }

        impl From<ElfProgramHeader> for ProgramHeader {
            fn from(ph: ElfProgramHeader) -> Self {
                ProgramHeader {
                    p_type   : ph.p_type,
                    p_flags  : ph.p_flags,
                    p_offset : ph.p_offset as $size,
                    p_vaddr  : ph.p_vaddr  as $size,
                    p_paddr  : ph.p_paddr  as $size,
                    p_filesz : ph.p_filesz as $size,
                    p_memsz  : ph.p_memsz  as $size,
                    p_align  : ph.p_align  as $size,
                }
            }
        }

        impl fmt::Debug for ProgramHeader {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f,
                       "p_type: {} p_flags 0x{:x} p_offset: 0x{:x} p_vaddr: 0x{:x} p_paddr: 0x{:x} \
                        p_filesz: 0x{:x} p_memsz: 0x{:x} p_align: {}",
                       pt_to_str(self.p_type),
                       self.p_flags,
                       self.p_offset,
                       self.p_vaddr,
                       self.p_paddr,
                       self.p_filesz,
                       self.p_memsz,
                       self.p_align)
            }
        }

        impl ProgramHeader {
            #[cfg(feature = "endian_fd")]
            pub fn parse(bytes: &[u8], mut offset: usize, count: usize, ctx: ::scroll::Endian) -> Result<Vec<ProgramHeader>> {
                use scroll::Pread;
                let mut program_headers = vec![ProgramHeader::default(); count];
                let offset = &mut offset;
                bytes.gread_inout_with(offset, &mut program_headers, ctx)?;
                Ok(program_headers)
            }

            pub fn from_bytes(bytes: &[u8], phnum: usize) -> Vec<ProgramHeader> {
                let mut phdrs = vec![ProgramHeader::default(); phnum];
                phdrs.copy_from_bytes(bytes).expect("buffer is too short for given number of entries");
                phdrs
            }

            pub unsafe fn from_raw_parts<'a>(phdrp: *const ProgramHeader,
                                             phnum: usize)
                                             -> &'a [ProgramHeader] {
                slice::from_raw_parts(phdrp, phnum)
            }

            pub fn from_fd(fd: &mut File, offset: u64, count: usize) -> Result<Vec<ProgramHeader>> {
                let mut phdrs = vec![ProgramHeader::default(); count];
                try!(fd.seek(Start(offset)));
                unsafe {
                    try!(fd.read(plain::as_mut_bytes(&mut *phdrs)));
                }
                Ok(phdrs)
            }
        }
    }
};}


pub mod program_header32 {
    pub use elf::program_header::*;

    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, Default)]
    #[cfg_attr(feature = "std", derive(Pread, Pwrite, SizeWith))]
    /// A 64-bit ProgramHeader typically specifies how to map executable and data segments into memory
    pub struct ProgramHeader {
        /// Segment type
        pub p_type: u32,
        /// Segment file offset
        pub p_offset: u32,
        /// Segment virtual address
        pub p_vaddr: u32,
        /// Segment physical address
        pub p_paddr: u32,
        /// Segment size in file
        pub p_filesz: u32,
        /// Segment size in memory
        pub p_memsz: u32,
        /// Segment flags
        pub p_flags: u32,
        /// Segment alignment
        pub p_align: u32,
    }

    pub const SIZEOF_PHDR: usize = 32;

    use plain;
    // Declare that this is a plain type.
    unsafe impl plain::Plain for ProgramHeader {}

    elf_program_header_std_impl!(u32);
}


pub mod program_header64 {
    pub use elf::program_header::*;

    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, Default)]
    #[cfg_attr(feature = "std", derive(Pread, Pwrite, SizeWith))]
    /// A 32-bit ProgramHeader typically specifies how to map executable and data segments into memory
    pub struct ProgramHeader {
        /// Segment type
        pub p_type: u32,
        /// Segment flags
        pub p_flags: u32,
        /// Segment file offset
        pub p_offset: u64,
        /// Segment virtual address
        pub p_vaddr: u64,
        /// Segment physical address
        pub p_paddr: u64,
        /// Segment size in file
        pub p_filesz: u64,
        /// Segment size in memory
        pub p_memsz: u64,
        /// Segment alignment
        pub p_align: u64,
    }

    pub const SIZEOF_PHDR: usize = 56;

    use plain;
    // Declare that this is a plain type.
    unsafe impl plain::Plain for ProgramHeader {}

    elf_program_header_std_impl!(u64);
}
