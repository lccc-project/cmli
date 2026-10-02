use std::{io::{ErrorKind, Result, Write}, num::NonZero};

use bitflags::bitflags;
use indexmap::IndexMap;

use crate::{instr::Instruction, intern::Symbol, mach::MachineMode, reloc::RelocValue};

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SectionData {
    Zeros(usize),
    Data(Vec<u8>),
    PermissionOnly,
    Note(Vec<Note>),
    Group(Symbol, Vec<Symbol>),
}

impl core::fmt::Display for SectionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SectionData::Zeros(n) => f.write_fmt(format_args!(".space {n}"))?,
            SectionData::Data(bytes) => {
                for b in bytes.chunks(16) {
                    f.write_str("\t")?;

                    for b in b {
                        f.write_fmt(format_args!("{b:02X} "))?;
                    }

                    f.write_str("\n")?;
                }
            },
            SectionData::PermissionOnly => {
                f.write_str("\tPermissions only\n")?;
            },
            SectionData::Note(notes) => {
                f.write_str("\tNote\n")?;
                for note in notes {
                    note.fmt(f)?;
                }
            },
            SectionData::Group(key, sections) => {
                f.write_fmt(format_args!("Group {key}"))?;

                for sect in sections {
                    f.write_fmt(format_args!("\t{sect}\n"))?;
                }
            },
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Note {
    pub name: Symbol,
    pub ty: u64,
    pub desc: Vec<u8>,
}

impl core::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("\t{} ({:#X}):\n", self.name, self.ty))?;

        for b in self.desc.chunks(16) {
            f.write_str("\t\t")?;

            for b in b {
                f.write_fmt(format_args!("{b:02X} "))?;
            }

            f.write_str("\n")?;
        }

        Ok(())
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    pub struct SectionPermissions : u16 {
        const READ = 0x01;
        const WRITE = 0x02;
        const EXEC = 0x04;
    }
}

impl core::fmt::Display for SectionPermissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("R")?;
        if self.contains(Self::WRITE) {
            f.write_str("W")?;
        }

        if self.contains(Self::EXEC) {
            f.write_str("X")?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum SectionType {
    Data(SectionPermissions),
    Bss,
    Note,
    Tls,
    Group,
    GnuStack(SectionPermissions),

    Custom(NonZero<u32>, SectionPermissions),
    CustomNoAlloc(NonZero<u32>),
}

impl core::fmt::Display for SectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SectionType::Data(perms) => f.write_fmt(format_args!("data {perms}")),
            SectionType::Bss => f.write_str("bss"),
            SectionType::Note => f.write_str("note"),
            SectionType::Tls => f.write_str("tls"),
            SectionType::Group => f.write_str("group"),
            SectionType::GnuStack(perms) => f.write_fmt(format_args!("stack {perms}")),
            SectionType::Custom(n, perms) => f.write_fmt(format_args!("custom type {n} {perms}")),
            SectionType::CustomNoAlloc(n) => f.write_fmt(format_args!("custom type {n} (no alloc)")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Section {
    name: Symbol,
    ty: SectionType,
    data: SectionData,
    relocs: Vec<(usize, RelocValue)>,
    dsize: usize,
    salign: usize,
}


impl core::fmt::Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {name, ty, data, relocs, salign, ..} = self;
        f.write_fmt(format_args!("section {name} {ty} align {salign}\n"))?;
        data.fmt(f)?;
        f.write_str("\n\n")
    }
}


impl Section {
    const fn create(name: Symbol, ty: SectionType, data: SectionData) -> Section {
        Self {name, ty, data, relocs: Vec::new(), dsize: 0, salign: 1}
    }

    pub const fn create_data(name: Symbol,p: SectionPermissions) -> Section {
        Self::create(name, SectionType::Data(p), SectionData::Data(Vec::new()))
    }

    pub const fn create_bss(name: Symbol) -> Section {
        Self::create(name, SectionType::Bss, SectionData::Zeros(0))
    }

    pub const fn create_tls(name: Symbol) -> Section {
        Self::create(name, SectionType::Tls, SectionData::Data(Vec::new()))
    }

    pub const fn create_tbss(name: Symbol) -> Section {
        Self::create(name, SectionType::Tls, SectionData::Zeros(0))
    }

    pub fn align_section(&mut self, align: usize) -> std::io::Result<()> {
        assert!(align.is_power_of_two());

        self.salign = self.salign.max(align);

        match &mut self.data {
            SectionData::Zeros(n) => {
                *n = (*n + (align - 1)) & !(align - 1);

                self.dsize = *n;
            },
            SectionData::Data(items) => {
                let nsize = (items.len() + (align - 1)) & (align - 1);

                items.resize(nsize, 0);

                self.dsize = nsize;
            },
            SectionData::PermissionOnly => {},
            SectionData::Note(notes) => Err(std::io::Error::new(ErrorKind::InvalidInput, "Cannot align a note section"))?,
            SectionData::Group(symbol, symbols) => Err(std::io::Error::new(ErrorKind::InvalidInput, "Cannot align a note section"))?,
        }

        Ok(())
    }

    pub fn write_zeros(&mut self, n: usize) -> std::io::Result<()> {
        match &mut self.data {
            SectionData::Zeros(zeros) => *zeros += n,
            SectionData::Data(items) => items.resize(items.len() + n, 0),
            SectionData::PermissionOnly => Err(std::io::Error::new(ErrorKind::WriteZero, "Cannot Write to a PermissionOnly section"))?,
            SectionData::Note(_) => Err(std::io::Error::new(ErrorKind::WriteZero, "Cannot Write to a Note section"))?,
            SectionData::Group(_, _) => Err(std::io::Error::new(ErrorKind::WriteZero, "Cannot Write to a Group section"))?,
        }

        self.dsize += n;

        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        match &mut self.data {
            SectionData::Data(data) => data.extend_from_slice(bytes),
            SectionData::Zeros(_) => Err(std::io::Error::new(ErrorKind::InvalidData, "Cannot write non-zero bytes to Zeros section"))?,
            SectionData::PermissionOnly => Err(std::io::Error::new(ErrorKind::WriteZero, "Cannot Write to a PermissionOnly section"))?,
            SectionData::Note(notes) => Err(std::io::Error::new(ErrorKind::WriteZero, "Cannot Write to a Note section"))?,
            SectionData::Group(symbol, symbols) => Err(std::io::Error::new(ErrorKind::WriteZero, "Cannot Write to a Group section"))?,
        }

        self.dsize += bytes.len();

        Ok(())
    }

    pub fn write_relocation(&mut self, reloff: isize, relval: RelocValue) -> std::io::Result<()> {
        let Some(off) = self.dsize.checked_add_signed(reloff) else {
            return Err(std::io::Error::new(ErrorKind::InvalidInput, format!("Offset {reloff} to section of size {} overflows", self.dsize)));
        };

        self.relocs.push((off, relval));

        Ok(())
    }

    pub fn require_permissions(&mut self, p: SectionPermissions) -> std::io::Result<()> {
        match &mut self.ty {
            SectionType::Data(perms) => {
                *perms |= p;
            },
            SectionType::Bss => {
                if p.contains(SectionPermissions::EXEC) {
                    return Err(std::io::Error::new(ErrorKind::InvalidInput, "Cannot make bss executable"));
                }
            },
            SectionType::Note => Err(std::io::Error::new(ErrorKind::InvalidInput, "Cannot change note section permissions"))?,
            SectionType::Tls => {
                if p.contains(SectionPermissions::EXEC) {
                    return Err(std::io::Error::new(ErrorKind::InvalidInput, "Cannot make tls section executable"));
                }
            },
            SectionType::Group => Err(std::io::Error::new(ErrorKind::InvalidInput, "Cannot change note section permissions"))?,
            SectionType::GnuStack(perms) => {
                *perms |= p;
            },
            SectionType::Custom(_, perms) => {
                *perms |= p;
            },
            SectionType::CustomNoAlloc(_) => Err(std::io::Error::new(ErrorKind::InvalidInput, "Cannot change non-allocatable section permissions"))?,
        }

        Ok(())
    }

    pub fn offset(&self) -> usize {
        self.dsize
    }
}


pub trait Encoder {
    fn encode_instr(&self, writer: &mut Section, instr: &Instruction, mode: MachineMode) -> Result<usize>;
    fn align_instruction(&self, writer: &mut Section) -> Result<()> {
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SymDef {
    pub offset: usize,
    pub section: Symbol,
    pub ty: SymbolType,
    pub size: usize,
    pub linkage: SymbolLinkage,
    pub visibility: SymbolVisibility,
    pub flags: u32,
}

impl core::fmt::Display for SymDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self{offset, section, ty, size, linkage, visibility, ..} = self;
        f.write_fmt(format_args!("{section} ({offset}) {ty} ({size} bytes), {linkage} ({visibility})"))
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SymbolType {
    None,
    Function,
    Object,
    Common,
    File,
    Section,
    Tls,
    IFunction,
    Custom(u8),
}

impl core::fmt::Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolType::None => f.write_str("no type"),
            SymbolType::Function => f.write_str("function"),
            SymbolType::Object => f.write_str("object"),
            SymbolType::Common => f.write_str("common"),
            SymbolType::File => f.write_str("file"),
            SymbolType::Section => f.write_str("section"),
            SymbolType::IFunction => f.write_str("ifunc resolver"),
            SymbolType::Tls => f.write_str("tls data"),
            SymbolType::Custom(n) => f.write_str("custom type {n}"),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SymbolLinkage {
    Local,
    Global,
    Weak,
    Custom(u8),
}

impl core::fmt::Display for SymbolLinkage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolLinkage::Local => f.write_str("local"),
            SymbolLinkage::Global => f.write_str("global"),
            SymbolLinkage::Weak => f.write_str("weak"),
            SymbolLinkage::Custom(n) => f.write_fmt(format_args!("custom linkage {n}")),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SymDecl {
    Def(SymDef),
    Extern(Option<SymbolVisibility>),
    ExternWeak(Option<SymbolVisibility>),
}

impl core::fmt::Display for SymDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymDecl::Def(def) => def.fmt(f),
            SymDecl::Extern(None) => f.write_str("extern"),
            SymDecl::Extern(Some(vis)) => f.write_fmt(format_args!("extern {vis}")),
            SymDecl::ExternWeak(None) => f.write_str("extern weak"),
            SymDecl::ExternWeak(Some(vis)) => f.write_fmt(format_args!("extern weak {vis}")),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SymbolVisibility {
    /// The default visibility for a global object.
    /// This has the semantics of either [`SymbolVisibility::Interposeable`] or [`SymbolVisibility::NotInterposeable`], depending on the format
    DefaultGlobal,
    Interposeable,
    NotInterposeable,
    DsoLocal,

    Custom(u8),
}

impl core::fmt::Display for SymbolVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolVisibility::DefaultGlobal => f.write_str("default"),
            SymbolVisibility::Interposeable => f.write_str("interposeable"),
            SymbolVisibility::NotInterposeable => f.write_str("protected"),
            SymbolVisibility::DsoLocal => f.write_str("hidden"),
            SymbolVisibility::Custom(n) => f.write_fmt(format_args!("custom visibility {n}")),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum FileType {
    Object,
    Executable,
    Dll,
    StaticLibrary,
    Core,
    Custom(u8),
}

impl core::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Object => f.write_str("rel"),
            FileType::Executable => f.write_str("exec"),
            FileType::Dll => f.write_str("dyn"),
            FileType::StaticLibrary => f.write_str("slib"),
            FileType::Core => f.write_str("core"),
            FileType::Custom(n) => f.write_fmt(format_args!("custom type {n}")),
        }
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct File {
    format: Symbol,
    file_type: FileType,
    sections: IndexMap<Symbol, Section>,
    symbols: IndexMap<Symbol, SymDecl>,
}

impl File {
    pub fn create(format: Symbol, file_type: FileType) -> File {
        File{format, file_type, sections: IndexMap::new(), symbols: IndexMap::new()}
    }

    pub fn get_or_create_section(&mut self, name: Symbol, f: impl FnOnce(Symbol) -> Section) -> &mut Section {
        if let Some(section) = self.sections.get_mut(&name) {
            // get_or_insert_with pattern
            unsafe { &mut *(section as *mut Section)}
        } else {
            let section = f(name);
            assert!(section.name == name);

            self.sections.insert(name, section);

            self.sections.get_mut(&name).unwrap()
        }
    }

    pub fn define_symbol(&mut self, name: Symbol, f: impl FnOnce()->SymDecl) -> &mut SymDecl {
        if let Some(sym) = self.symbols.get_mut(&name) {
            match sym {
                SymDecl::Def(sym_def) => {},
                _ => *sym = f(),
            }
            unsafe { &mut *(sym as *mut SymDecl)}
        } else {
            let sym = f();
            self.symbols.insert(name, sym);

            self.symbols.get_mut(&name).unwrap()
        }
    }
}

impl core::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} \"{}\"\n", self.file_type, self.format))?;

        for (sym, decl) in &self.symbols {
            f.write_fmt(format_args!("\t{sym}: {decl}\n"))?;
        }

        for (_, section) in &self.sections {
            section.fmt(f)?;
        }

        Ok(())

    }
}