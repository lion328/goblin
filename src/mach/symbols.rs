//! "Nlist" style symbols in this binary - beware, like most symbol tables in most binary formats, they are strippable, and should not be relied upon, see the imports and exports modules for something more permanent.
//!
//! Symbols are essentially a type, offset, and the symbol name

use scroll::{ctx, Pread};
use scroll::ctx::SizeWith;
use error;
use container::{self, Container};
use mach::load_command;
use core::fmt::{self, Debug};

// The n_type field really contains four fields which are used via the following masks.
/// if any of these bits set, a symbolic debugging entry
pub const N_STAB: u8 = 0xe0;
/// private external symbol bit
pub const N_PEXT: u8 = 0x10;
/// mask for the type bits
pub const N_TYPE: u8 = 0x0e;
/// external symbol bit, set for external symbols
pub const N_EXT: u8 = 0x01;

// If the type is N_SECT then the n_sect field contains an ordinal of the
// section the symbol is defined in.  The sections are numbered from 1 and
// refer to sections in order they appear in the load commands for the file
// they are in.  This means the same ordinal may very well refer to different
// sections in different files.

// The n_value field for all symbol table entries (including N_STAB's) gets
// updated by the link editor based on the value of it's n_sect field and where
// the section n_sect references gets relocated.  If the value of the n_sect
// field is NO_SECT then it's n_value field is not changed by the link editor.
/// symbol is not in any section
pub const NO_SECT: u8 = 0;
/// 1 thru 255 inclusive
pub const MAX_SECT: u8 = 255;

/// undefined, n_sect == NO_SECT
pub const N_UNDF: u8 = 0x0;
/// absolute, n_sect == NO_SECT
pub const N_ABS:  u8 = 0x2;
/// defined in section number n_sect
pub const N_SECT: u8 = 0xe;
/// prebound undefined (defined in a dylib)
pub const N_PBUD: u8 = 0xc;
/// indirect
pub const N_INDR: u8 = 0xa;

pub const NLIST_TYPE_MASK: u8 = 0xe;
pub const NLIST_TYPE_GLOBAL: u8 = 0x1;
pub const NLIST_TYPE_LOCAL: u8 = 0x0;

pub fn n_type_to_str(n_type: u8) -> &'static str {
    match n_type {
        N_UNDF => "N_UNDF",
        N_ABS => "N_ABS",
        N_SECT => "N_SECT",
        N_PBUD => "N_PBUD",
        N_INDR => "N_INDR",
        _ => "UNKNOWN_N_TYPE"
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pread, Pwrite, SizeWith)]
pub struct Nlist32 {
    /// index into the string table
    pub n_strx: u32,
    /// type flag, see below
    pub n_type: u8,
    /// section number or NO_SECT
    pub n_sect: u8,
    /// see <mach-o/stab.h>
    pub n_desc: u16,
    /// value of this symbol (or stab offset)
    pub n_value: u32,
}

pub const SIZEOF_NLIST_32: usize = 12;

impl Debug for Nlist32 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "strx: {:04} type: {:#02x} sect: {:#x} desc: {:#03x} value: {:#x}",
               self.n_strx,
               self.n_type,
               self.n_sect,
               self.n_desc,
               self.n_value,
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pread, Pwrite, SizeWith)]
pub struct Nlist64 {
    /// index into the string table
    pub n_strx: u32,
    /// type flag, see below
    pub n_type: u8,
    /// section number or NO_SECT
    pub n_sect: u8,
    /// see <mach-o/stab.h>
    pub n_desc: u16,
    /// value of this symbol (or stab offset)
    pub n_value: u64,
}

pub const SIZEOF_NLIST_64: usize = 16;

impl Debug for Nlist64 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "strx: {:04} type: {:#02x} sect: {:#x} desc: {:#03x} value: {:#x}",
               self.n_strx,
               self.n_type,
               self.n_sect,
               self.n_desc,
               self.n_value,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Nlist {
    /// index into the string table
    pub n_strx: usize,
    /// type flag, see below
    pub n_type: u8,
    /// section number or NO_SECT
    pub n_sect: usize,
    /// see <mach-o/stab.h>
    pub n_desc: u16,
    /// value of this symbol (or stab offset)
    pub n_value: u64,
}

impl Nlist {
    /// Gets this symbol's type in bits 0xe
    pub fn get_type(&self) -> u8 {
        self.n_type & N_TYPE
    }
    /// Gets the str representation of the type of this symbol
    pub fn type_str(&self) -> &'static str {
        n_type_to_str(self.get_type())
    }
    /// Whether this symbol is global or not
    pub fn is_global(&self) -> bool {
        self.n_type & N_EXT != 0
    }
    /// Whether this symbol is undefined or not
    pub fn is_undefined(&self) -> bool {
        self.n_sect == 0 && self.n_type & N_TYPE == N_UNDF
    }
    /// Whether this symbol is a symbolic debugging entry
    pub fn is_stab(&self) -> bool {
        self.n_type & N_STAB != 0
    }
}

impl ctx::SizeWith<container::Ctx> for Nlist {
    type Units = usize;
    fn size_with(ctx: &container::Ctx) -> usize {
        use container::Container;
        match ctx.container {
            Container::Little => {
                SIZEOF_NLIST_32
            },
            Container::Big => {
                SIZEOF_NLIST_64
            },
        }
    }
}

impl From<Nlist32> for Nlist {
    fn from(nlist: Nlist32) -> Self {
        Nlist {
            n_strx: nlist.n_strx as usize,
            n_type: nlist.n_type,
            n_sect: nlist.n_sect as usize,
            n_desc: nlist.n_desc,
            n_value: nlist.n_value as u64,
        }
    }
}

impl From<Nlist64> for Nlist {
    fn from(nlist: Nlist64) -> Self {
        Nlist {
            n_strx: nlist.n_strx as usize,
            n_type: nlist.n_type,
            n_sect: nlist.n_sect as usize,
            n_desc: nlist.n_desc,
            n_value: nlist.n_value,
        }
    }
}

impl<'a> ctx::TryFromCtx<'a, container::Ctx> for Nlist {
    type Error = ::error::Error;
    type Size = usize;
    fn try_from_ctx(bytes: &'a [u8], container::Ctx { container, le }: container::Ctx) -> ::error::Result<(Self, Self::Size)> {
        let nlist = match container {
            Container::Little => {
                (bytes.pread_with::<Nlist32>(0, le)?.into(), SIZEOF_NLIST_32)
            },
            Container::Big => {
                (bytes.pread_with::<Nlist64>(0, le)?.into(), SIZEOF_NLIST_64)
            },
        };
        Ok(nlist)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SymbolsCtx {
    pub nsyms: usize,
    pub strtab: usize,
    pub ctx: container::Ctx,
}

impl<'a, T: ?Sized> ctx::TryFromCtx<'a, SymbolsCtx, T> for Symbols<'a> where T: AsRef<[u8]> {
    type Error = ::error::Error;
    type Size = usize;
    fn try_from_ctx(bytes: &'a T, SymbolsCtx {
        nsyms, strtab, ctx
    }: SymbolsCtx) -> ::error::Result<(Self, Self::Size)> {
        let data = bytes.as_ref();
        Ok ((Symbols {
            data: data,
            start: 0,
            nsyms: nsyms,
            strtab: strtab,
            ctx: ctx,
        }, data.len()))
    }
}

#[derive(Default)]
pub struct SymbolIterator<'a> {
    data: &'a [u8],
    nsyms: usize,
    offset: usize,
    count: usize,
    ctx: container::Ctx,
    strtab: usize,
}

impl<'a> Iterator for SymbolIterator<'a> {
    type Item = error::Result<(&'a str, Nlist)>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.nsyms {
            None
        } else {
            self.count += 1;
            match self.data.gread_with::<Nlist>(&mut self.offset, self.ctx) {
                Ok(symbol) => {
                    match self.data.pread(self.strtab + symbol.n_strx) {
                        Ok(name) => {
                            Some(Ok((name, symbol)))
                        },
                        Err(e) => return Some(Err(e.into()))
                    }
                },
                Err(e) => return Some(Err(e.into()))
            }
        }
    }
}

/// A zero-copy "nlist" style symbol table ("stab"), including the string table
pub struct Symbols<'a> {
    data: &'a [u8],
    start: usize,
    nsyms: usize,
    // TODO: we can use an actual strtab here and tie it to symbols lifetime
    strtab: usize,
    ctx: container::Ctx,
}

impl<'a, 'b> IntoIterator for &'b Symbols<'a> {
    type Item = <SymbolIterator<'a> as Iterator>::Item;
    type IntoIter = SymbolIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> Symbols<'a> {
    /// Creates a new symbol table with `count` elements, from the `start` offset, using the string table at `strtab`, with a _default_ ctx.
    ////
    /// **Beware**, this will provide incorrect results if you construct this on a 32-bit mach binary, using a 64-bit machine; use `parse` instead if you want 32/64 bit support
    pub fn new(bytes: &'a [u8], start: usize, count: usize, strtab: usize) -> error::Result<Symbols<'a>> {
        let nsyms = count;
        Ok (Symbols {
            data: bytes,
            start: start,
            nsyms: nsyms,
            strtab: strtab,
            ctx: container::Ctx::default(),
        })
    }
    pub fn parse(bytes: &'a [u8], symtab: &load_command::SymtabCommand, ctx: container::Ctx) -> error::Result<Symbols<'a>> {
        // we need to normalize the strtab offset before we receive the truncated bytes in pread_with
        let strtab = symtab.stroff - symtab.symoff;
        Ok(bytes.pread_with(symtab.symoff as usize, SymbolsCtx { nsyms: symtab.nsyms as usize, strtab: strtab as usize, ctx: ctx })?)
    }

    pub fn iter(&self) -> SymbolIterator<'a> {
        SymbolIterator {
            offset: self.start as usize,
            nsyms: self.nsyms as usize,
            count: 0,
            data: self.data,
            ctx: self.ctx,
            strtab: self.strtab,
        }
    }

    /// Parses a single Nlist symbol from the binary, with its accompanying name
    pub fn get(&self, index: usize) -> ::error::Result<(&'a str, Nlist)> {
        let sym: Nlist = self.data.pread_with(self.start + (index * Nlist::size_with(&self.ctx)), self.ctx)?;
        let name = self.data.pread(self.strtab + sym.n_strx)?;
        Ok((name, sym))
    }
}

impl<'a> Debug for Symbols<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "Data: {} start: {:#?}, nsyms: {} strtab: {:#x}", self.data.len(), self.start, self.nsyms, self.strtab)?;
        writeln!(fmt, "Symbols: {{")?;
        for (i, res) in self.iter().enumerate() {
            match res {
                Ok((name, nlist)) => {
                    writeln!(fmt, "{: >10x} {} sect: {:#x} type: {:#02x} desc: {:#03x}", nlist.n_value, name, nlist.n_sect, nlist.n_type, nlist.n_desc)?;
                },
                Err(error) => {
                    writeln!(fmt, "  Bad symbol, index: {}, sym: {:?}", i, error)?;
                }
            }
        }
        writeln!(fmt, "}}")
    }
}
