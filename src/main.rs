use clap::{Parser, CommandFactory};
use check::{dump_header, read_header};
use load::{load_segment, read_phdr, ElfPhdr, dump_phdrs, dump_memory};
use std::{io::Cursor, mem::size_of, process::exit};

use crate::disas::{disassemble_code, disassemble_data, disassemble_rodata};

mod check;
mod load;
mod disas;

pub const MEMSIZE: u16 = 1 << 12;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Show the Mini-ELF header
    #[arg(short = 'H')]
    hdr: bool,
 
    /// Show all with brief memory
    #[arg(short = 'a')]
    all_brief: bool,
   
    /// Show all with full memory
    #[arg(short = 'f')]
    all_full: bool,
   
    /// Show the program headers
    #[arg(short = 's')]
    phdrs: bool,
    
    /// Show the memory contents (brief)
    #[arg(short = 'm')]
    mem_brief: bool,
   
    /// Show the memory contents (full)
    #[arg(short = 'M')]
    mem_full: bool,

    /// Disassemble code contents
    #[arg(short = 'd')]
    disas_code: bool,

    /// Disassemble data contents
    #[arg(short = 'D')]
    disas_data: bool,

    /// Mini-ELF object file
    file: String,
}

fn fail() -> ! {
    println!("Failed to read file");
    exit(1);
}

fn process_args(args: &mut Args) -> bool {
    if args.all_brief {
        args.hdr = true;
        args.phdrs = true;
        args.mem_brief = true;
    }
    if args.all_full {
        args.hdr = true;
        args.phdrs = true;
        args.mem_full = true;
    }

    if args.mem_brief && args.mem_full {
        return false;
    }

    true
}

fn main() {
    let mut args = Args::parse();
    if !process_args(&mut args) {
        Args::command().print_help().unwrap();
        exit(0);
    }

    let bytes = match std::fs::read(args.file) {
        Ok(v) => v,
        Err(_) => fail(),
    };
    let mut reader = Cursor::new(bytes);

    // load the header
    let hdr = match read_header(&mut reader) {
        Some(v) => v,
        None => fail(),
    };

    // load the program headers
    let mut phdrs: Vec<ElfPhdr> = Vec::with_capacity(hdr.num_phdr as usize);
    for i in 0..hdr.num_phdr {
        let offset: u16 = hdr.phdr_start + size_of::<ElfPhdr>() as u16 * i;
        let phdr = match read_phdr(&mut reader, offset) {
            Some(v) => v,
            None => fail(),
        };
        phdrs.push(phdr);
    }

    // load all segments into virtual memory
    let mut memory: Box<[u8]> = Box::new([0; MEMSIZE as usize]);
    for phdr in phdrs.iter() {
        if !load_segment(&mut reader, &mut memory, phdr) {
            fail();
        }
    }

    if args.hdr {
        dump_header(&hdr);
    }

    if args.phdrs {
        dump_phdrs(&phdrs);
    }

    if args.mem_full {
        dump_memory(&memory, 0, MEMSIZE);
    }

    if args.mem_brief {
        for phdr in phdrs.iter() {
            dump_memory(&memory, phdr.vaddr as u16, (phdr.vaddr + phdr.size) as u16);
        }
    }

    if args.disas_code {
        println!("Disassembly of executable contents:");
        for phdr in phdrs.iter() {
            if phdr.ptype == 1 {
                disassemble_code(&memory, phdr, &hdr);
            }
        }
    }

    if args.disas_data {
        println!("Disassembly of data contents:");
        for phdr in phdrs.iter() {
            if phdr.ptype == 0 {
                if phdr.flags == 4 {
                    disassemble_rodata(&memory, phdr);
                } else {
                    disassemble_data(&memory, phdr);
                }
            }
        }
    }
}
