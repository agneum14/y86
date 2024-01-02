use std::io::{Cursor, Read};

use binrw::BinRead;

const MAGIC: u32 = 0xdeadbeef;

#[repr(C)]
#[derive(BinRead)]
pub struct ElfPhdr {
    pub offset: u32,
    pub size: u32,
    pub vaddr: u32,
    pub ptype: u16,
    pub flags: u16,
    pub magic: u32,
}

pub fn read_phdr(reader: &mut Cursor<Vec<u8>>, offset: u16) -> Option<ElfPhdr> {
    reader.set_position(offset.into());
    if reader.position() != offset.into() {
        return None;
    }

    let phdr = match ElfPhdr::read_le(reader) {
        Ok(v) => v,
        Err(_) => return None,
    };

    if phdr.magic != MAGIC {
        return None;
    }
    Some(phdr)
}

pub fn load_segment(reader: &mut Cursor<Vec<u8>>, memory: &mut Box<[u8]>, phdr: &ElfPhdr) -> bool {
    reader.set_position(phdr.offset as u64);
    if reader.position() != phdr.offset as u64 {
        return false;
    }

    let mut buf = vec![0u8; phdr.size as usize];
    if reader.read_exact(&mut buf).is_err() {
        return false;
    }

    for (i, buf_byte) in buf.iter().enumerate() {
        let mem_byte = match memory.get_mut(phdr.vaddr as usize + i) {
            Some(v) => v,
            None => return false,
        };
        *mem_byte = *buf_byte;
    }

    true
}

pub fn dump_phdrs(phdrs: &Vec<ElfPhdr>) {
    println!(" Segment   Offset    Size      VirtAddr  Type      Flags");

    for (i, phdr) in phdrs.iter().enumerate() {
        print!(
            "  {:02x}       0x{:04x}    0x{:04x}    0x{:04x}",
            i, phdr.offset, phdr.size, phdr.vaddr
        );

        // print type
        print!("    ");
        match phdr.ptype {
            0 => print!("DATA "),
            1 => print!("CODE "),
            _ => print!("STACK"),
        }

        // print flags
        print!("     ");
        if phdr.flags >> 2 == 1 {
            print!("R");
        } else {
            print!(" ");
        }
        if phdr.flags >> 1 & 1 == 1 {
            print!("W");
        } else {
            print!(" ");
        }
        if phdr.flags & 1 == 1 {
            print!("X");
        } else {
            print!(" ");
        }
        println!();
    }
}

pub fn dump_memory(memory: &Box<[u8]>, start: u16, end: u16) {
    print!("Contents of memory from {:04x} to {:04x}:", start, end);

    // floor address for unaligned memory
    let addr = start & 0xFFF0;

    let mut i = 0;
    while addr + i < end {
        let byte = memory.get(i as usize).unwrap();

        if i % 16 == 0 {
            print!("\n  {:04x}  ", addr + i);
        } else if i % 8 == 0 {
            print!("  ");
        } else {
            print!(" ");
        }

        // print spaces before start address
        if addr + i < start {
            print!("  ");
        } else {
            print!("{:02x}", byte);
        }

        i += 1;
    }
    println!();
}
