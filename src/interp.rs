use std::io::Cursor;

use binrw::BinRead;

const MAGIC: u32 = 0x464c45;

#[repr(C)]
#[derive(BinRead)]
pub struct ElfHdr {
    pub version: u16,
    pub entry: u16,
    pub phdr_start: u16,
    pub num_phdr: u16,
    pub symtab: u16,
    pub strtab: u16,
    pub magic: u32,
}

pub fn read_header(reader: &mut Cursor<Vec<u8>>) -> Option<ElfHdr> {
    let hdr = match ElfHdr::read_le(reader) {
        Ok(v) => v,
        Err(_) => return None,
    };

    if hdr.magic != MAGIC {
        return None;
    }
    Some(hdr)
}

trait Stringify {
    fn stringify(&self) -> String;
}

impl Stringify for u16 {
    fn stringify(&self) -> String {
        format!("{:02x} {:02x}", self & 0x00FF, self >> 8)
    }
}
pub fn dump_header(hdr: &ElfHdr) {
    println!(
        "{} {} {} {}  {} {} 45 4c 46 00",
        hdr.version.stringify(),
        hdr.entry.stringify(),
        hdr.phdr_start.stringify(),
        hdr.num_phdr.stringify(),
        hdr.symtab.stringify(),
        hdr.strtab.stringify()
    );
    println!("Mini-ELF version {}", hdr.version);
    println!("Entry point 0x{:x}", hdr.entry);
    println!(
        "There are {} program headers, starting at offset {} (0x{:x})",
        hdr.num_phdr, hdr.phdr_start, hdr.phdr_start
    );

    if hdr.symtab == 0 {
        println!("There is no symbol table present");
    } else {
        println!(
            "There is a symbol table starting at offset {} (0x{:x})",
            hdr.symtab, hdr.symtab
        );
    }

    if hdr.strtab == 0 {
        println!("There is no string table present");
    } else {
        println!(
            "There is a string table starting at offset {} (0x{:x})",
            hdr.strtab, hdr.strtab
        );
    }
}
