use std::io::{Cursor, Read};

use anyhow::{ensure, Context, Result};
use binrw::BinRead;

use crate::error::mem_access;

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

pub fn read_phdr(reader: &mut Cursor<Vec<u8>>, offset: u16) -> Result<ElfPhdr> {
    reader.set_position(offset.into());
    ensure!(reader.position() == offset.into());

    let phdr = ElfPhdr::read_le(reader)?;
    ensure!(phdr.magic == MAGIC);

    Ok(phdr)
}

pub fn load_segment(
    reader: &mut Cursor<Vec<u8>>,
    memory: &mut Box<[u8]>,
    phdr: &ElfPhdr,
) -> Result<()> {
    reader.set_position(phdr.offset as u64);
    ensure!(reader.position() == phdr.offset as u64);

    let mut buf = vec![0u8; phdr.size as usize];
    reader.read_exact(&mut buf)?;

    for (i, buf_byte) in buf.iter().enumerate() {
        let addr: usize = phdr.vaddr as usize + i;
        let mem_byte = memory.get_mut(addr).context(mem_access(addr))?;
        *mem_byte = *buf_byte;
    }

    Ok(())
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

pub fn dump_memory(memory: &Box<[u8]>, start: u16, end: u16) -> Result<()> {
    print!("Contents of memory from {:04x} to {:04x}:", start, end);

    // floor address for unaligned memory
    let addr = start & 0xFFF0;

    let mut i = 0;
    while addr + i < end {
        let byte = memory.get(i as usize).context(mem_access(i as usize))?;

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

    Ok(())
}
