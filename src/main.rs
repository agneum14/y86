use std::{io::Cursor, process::exit};

use clap::Parser;
use interp::{read_header, dump_header};

mod interp;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Show the Mini-ELF header
    #[arg(short = 'H')]
    print_hdr: bool,

    /// Mini-ELF object file
    file: String
}

fn main() {
    let args = Args::parse();

    let bytes = match std::fs::read(args.file) {
        Ok(v) => v,
        Err(_) => {
            println!("Failed to read file");
            exit(1);
        }
    };
    let mut reader = Cursor::new(bytes);

    let hdr = match read_header(&mut reader) {
        Some(v) => v,
        None => {
            println!("Failed to read file");
            exit(2);
        }
    };

    if args.print_hdr {
        dump_header(&hdr);
    }
}
