use std::process::exit;

use crate::{interp::ElfHdr, load::ElfPhdr};

const NUM_REGS: u8 = 15;

type Address = u64;
type Flag = bool;

enum Stat {
    AOK,
    HLT,
    ADR,
    INS,
}

pub struct Cpu {
    reg: [Address; NUM_REGS as usize],
    zf: Flag,
    sf: Flag,
    of: Flag,
    pc: Address,
    stat: Stat,
}

#[derive(PartialEq)]
enum Icode {
    HALT,
    NOP,
    CMOV,
    IRMOVQ,
    RMMOVQ,
    MRMOVQ,
    OPQ,
    JUMP,
    CALL,
    RET,
    PUSHQ,
    POPQ,
    INVALID,
}

impl Icode {
    fn from(val: u8) -> Icode {
        match val {
            0 => Icode::HALT,
            1 => Icode::NOP,
            2 => Icode::CMOV,
            3 => Icode::IRMOVQ,
            4 => Icode::RMMOVQ,
            5 => Icode::MRMOVQ,
            6 => Icode::OPQ,
            7 => Icode::JUMP,
            8 => Icode::CALL,
            9 => Icode::RET,
            10 => Icode::PUSHQ,
            11 => Icode::POPQ,
            _ => Icode::INVALID,
        }
    }
}

#[derive(PartialEq)]
enum Register {
    RAX,
    RCX,
    RDX,
    RBX,
    RSP,
    RBP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    NOREG,
}

impl Register {
    fn set(&mut self, val: u8) -> bool {
        *self = match val {
            0 => Register::RAX,
            1 => Register::RCX,
            2 => Register::RDX,
            3 => Register::RBX,
            4 => Register::RSP,
            5 => Register::RBP,
            6 => Register::RSI,
            7 => Register::RDI,
            8 => Register::R8,
            9 => Register::R9,
            10 => Register::R10,
            11 => Register::R11,
            12 => Register::R12,
            13 => Register::R13,
            14 => Register::R14,
            _ => return false,
        };

        true
    }

    fn set_f(&mut self, val: u8) -> bool {
        if val != 15 {
            return false;
        }
        *self = Register::NOREG;

        true
    }

    fn set_a(&mut self, val: u8) -> bool {
        if val <= 14 {
            self.set(val);
        } else if val == 15 {
            self.set_f(val);
        } else {
            return false;
        }

        true
    }

    fn to_string(&self) -> String {
        match self {
            Register::RAX => String::from("rax"),
            Register::RCX => String::from("rcx"),
            Register::RDX => String::from("rdx"),
            Register::RBX => String::from("rbx"),
            Register::RSP => String::from("rsp"),
            Register::RBP => String::from("rbp"),
            Register::RSI => String::from("rsi"),
            Register::RDI => String::from("rdi"),
            Register::R8 => String::from("r8"),
            Register::R9 => String::from("r9"),
            Register::R10 => String::from("r10"),
            Register::R11 => String::from("r11"),
            Register::R12 => String::from("r12"),
            Register::R13 => String::from("r13"),
            Register::R14 => String::from("r14"),
            Register::NOREG => String::from(""),
        }
    }
}

pub struct Inst {
    icode: Icode,
    ifun: u8,
    ra: Register,
    rb: Register,
    val_c: Option<Address>,
    val_p: Address,
}

enum Cmov {
    RRMOVQ,
    CMOVLE,
    CMOVL,
    CMOVE,
    CMOVNE,
    CMOVGE,
    CMOVG,
}

impl Cmov {
    pub fn from(val: u8) -> Cmov {
        match val {
            0 => Cmov::RRMOVQ,
            1 => Cmov::CMOVLE,
            2 => Cmov::CMOVL,
            3 => Cmov::CMOVE,
            4 => Cmov::CMOVNE,
            5 => Cmov::CMOVGE,
            _ => Cmov::CMOVG,
        }
    }
}

enum Opq {
    ADD,
    SUB,
    AND,
    XOR,
}

impl Opq {
    fn from(val: u8) -> Opq {
        match val {
            0 => Opq::ADD,
            1 => Opq::SUB,
            2 => Opq::AND,
            _ => Opq::XOR,
        }
    }
}

enum Jump {
    JMP,
    JLE,
    JL,
    JE,
    JNE,
    JGE,
    JG,
}

impl Jump {
    fn from(val: u8) -> Jump {
        match val {
            0 => Jump::JMP,
            1 => Jump::JLE,
            2 => Jump::JL,
            3 => Jump::JE,
            4 => Jump::JNE,
            5 => Jump::JGE,
            _ => Jump::JG,
        }
    }
}

pub fn fetch(cpu: &mut Cpu, memory: &Box<[u8]>) -> Inst {
    let mut inst = Inst {
        icode: Icode::INVALID,
        ifun: 0,
        ra: Register::NOREG,
        rb: Register::NOREG,
        val_c: None,
        val_p: 0,
    };

    let b0 = match memory.get(cpu.pc as usize) {
        Some(v) => v,
        None => {
            inv_mem(&mut inst, cpu);
            return inst;
        }
    };
    inst.icode = Icode::from(b0 >> 4);
    inst.ifun = b0 & 0xF;

    // set valP
    inst.val_p = cpu.pc;
    match inst.icode {
        Icode::HALT | Icode::NOP | Icode::RET => inst.val_p += 1,
        Icode::CMOV | Icode::OPQ | Icode::PUSHQ | Icode::POPQ => inst.val_p += 2,
        Icode::JUMP | Icode::CALL => inst.val_p += 9,
        Icode::IRMOVQ | Icode::RMMOVQ | Icode::MRMOVQ => inst.val_p += 10,
        Icode::INVALID => {
            inst.ifun = *b0;
            inv_inst(&mut inst, cpu);
            return inst;
        }
    }

    // set second byte if required
    let (mut b1_h, mut b1_l): (u8, u8) = (0, 0);
    if inst.val_p - cpu.pc > 1 {
        let b1 = match memory.get((cpu.pc + 1) as usize) {
            Some(v) => v,
            None => {
                inv_mem(&mut inst, cpu);
                return inst;
            }
        };
        b1_h = b1 >> 4;
        b1_l = b1 & 0xF;
    }

    // check and set every instruction
    match inst.icode {
        Icode::HALT => {
            if inst.ifun != 0 {
                inv_inst(&mut inst, cpu);
                return inst;
            }
            cpu.stat = Stat::HLT;
        }
        Icode::NOP | Icode::RET => {
            if inst.ifun != 0 {
                inv_inst(&mut inst, cpu);
                return inst;
            }
        }
        Icode::CMOV => {
            if inst.ifun > 6 || !inst.ra.set(b1_h) || !inst.rb.set(b1_l) {
                inv_inst(&mut inst, cpu);
                return inst;
            }
        }
        Icode::IRMOVQ => {
            inst.val_c = memtoi(memory, cpu.pc + 2);
            if inst.ifun != 0 || !inst.ra.set_f(b1_h) || !inst.rb.set(b1_l) || inst.val_c.is_none()
            {
                inv_inst(&mut inst, cpu);
                return inst;
            }
        }
        Icode::RMMOVQ | Icode::MRMOVQ => {
            inst.val_c = memtoi(memory, cpu.pc + 2);
            if inst.ifun != 0 || !inst.ra.set(b1_h) || !inst.rb.set_a(b1_l) || inst.val_c.is_none()
            {
                inv_inst(&mut inst, cpu);
                return inst;
            }
        }
        Icode::OPQ => {
            if inst.ifun > 3 || !inst.ra.set(b1_h) || !inst.rb.set(b1_l) {
                inv_inst(&mut inst, cpu);
                return inst;
            }
        }
        Icode::JUMP => {
            inst.val_c = memtoi(memory, cpu.pc + 1);
            if inst.ifun > 6 || inst.val_c.is_none() {
                inv_inst(&mut inst, cpu);
                return inst;
            }
        }
        Icode::CALL => {
            inst.val_c = memtoi(memory, cpu.pc + 1);
            if inst.ifun != 0 || inst.val_c.is_none() {
                inv_inst(&mut inst, cpu);
                return inst;
            }
        }
        Icode::PUSHQ | Icode::POPQ => {
            if inst.ifun != 0 || !inst.ra.set(b1_h) || !inst.rb.set_f(b1_l) {
                inv_inst(&mut inst, cpu);
                return inst;
            }
        }
        Icode::INVALID => (), // impossible
    }

    inst
}

pub fn disassemble(inst: &Inst) {
    match inst.icode {
        Icode::HALT => print!("halt"),
        Icode::NOP => print!("nop"),
        Icode::CMOV => {
            match Cmov::from(inst.ifun) {
                Cmov::RRMOVQ => print!("rrmovq "),
                Cmov::CMOVLE => print!("cmovle "),
                Cmov::CMOVL => print!("cmovl "),
                Cmov::CMOVE => print!("cmove "),
                Cmov::CMOVNE => print!("cmovne "),
                Cmov::CMOVGE => print!("cmovge "),
                Cmov::CMOVG => print!("cmovg "),
            }

            print!("%{}, %{}", inst.ra.to_string(), inst.rb.to_string());
        }
        Icode::IRMOVQ => print!(
            "irmovq 0x{:x}, %{}",
            inst.val_c.unwrap(),
            inst.rb.to_string()
        ),
        Icode::RMMOVQ => {
            print!(
                "rmmovq %{}, 0x{:x}",
                inst.ra.to_string(),
                inst.val_c.unwrap()
            );
            if inst.rb != Register::NOREG {
                print!("(%{})", inst.rb.to_string());
            }
        }
        Icode::MRMOVQ => {
            print!("mrmovq 0x{:x}", inst.val_c.unwrap());
            if inst.rb != Register::NOREG {
                print!("(%{})", inst.rb.to_string());
            }
            print!(", %{}", inst.ra.to_string());
        }
        Icode::OPQ => {
            match Opq::from(inst.ifun) {
                Opq::ADD => print!("add"),
                Opq::SUB => print!("sub"),
                Opq::AND => print!("and"),
                Opq::XOR => print!("xor"),
            }

            print!("q %{}, %{}", inst.ra.to_string(), inst.rb.to_string());
        }
        Icode::JUMP => {
            match Jump::from(inst.ifun) {
                Jump::JMP => print!("jmp"),
                Jump::JLE => print!("jle"),
                Jump::JL => print!("jl"),
                Jump::JE => print!("je"),
                Jump::JNE => print!("jne"),
                Jump::JGE => print!("jge"),
                Jump::JG => print!("jg"),
            }

            print!(" 0x{:x}", inst.val_c.unwrap());
        }
        Icode::CALL => print!("call 0x{:x}", inst.val_c.unwrap()),
        Icode::RET => print!("ret"),
        Icode::PUSHQ => print!("pushq %{}", inst.ra.to_string()),
        Icode::POPQ => print!("popq %{}", inst.ra.to_string()),
        Icode::INVALID => (), // impossible
    }
}

pub fn disassemble_code(memory: &Box<[u8]>, phdr: &ElfPhdr, hdr: &ElfHdr) {
    // fake cpu to hold pc
    let mut cpu = Cpu {
        reg: [0u64; NUM_REGS as usize],
        zf: false,
        of: false,
        sf: false,
        pc: phdr.vaddr as u64,
        stat: Stat::AOK,
    };

    println!(
        "  0x{:03x}:                               | .pos 0x{:03x} code",
        phdr.vaddr, phdr.vaddr
    );

    let end = phdr.vaddr + phdr.size;
    while cpu.pc < end as u64 {
        if cpu.pc == hdr.entry as u64 {
            println!(
                "  0x{:03x}:                               | _start:",
                cpu.pc
            );
        }

        // abort with error if instruction is invalid
        let inst = fetch(&mut cpu, memory);
        if inst.icode == Icode::INVALID {
            println!("Invalid opcode: 0x{:x}\n", inst.ifun);
            exit(1);
        }

        // print current address and raw bytes of instruction
        print!("  0x{:03x}: ", cpu.pc);
        for i in 0..10 {
            let byte = memory.get((cpu.pc + i) as usize).unwrap();
            if i < inst.val_p - cpu.pc {
                print!("{:02x} ", byte);
            } else {
                print!("   ");
            }
        }
        print!("|   ");

        disassemble(&inst);
        println!();
        cpu.pc = inst.val_p;
    }

    println!();
}

pub fn disassemble_data(memory: &Box<[u8]>, phdr: &ElfPhdr) {
    let mut addr = phdr.vaddr;
    let max_addr = addr + phdr.size;

    println!(
        "  0x{:03x}:                               | .pos 0x{:03x} data",
        phdr.vaddr, phdr.vaddr
    );

    while addr < max_addr {
        print!("  0x{:03x}: ", addr);
        for i in 0..8 {
            let byte = memory.get((addr + i) as usize).unwrap(); // memory must be valid, checked
                                                                 // in load_segment
            print!("{:02x} ", byte);
        }
        println!(
            "      |   .quad 0x{:x}",
            memtoi(memory, addr as u64).unwrap()
        );

        addr += 8;
    }

    println!();
}

pub fn disassemble_rodata(memory: &Box<[u8]>, phdr: &ElfPhdr) {
    let mut addr = phdr.vaddr;
    let max_addr = phdr.vaddr + phdr.size;
    let mut s: String = String::from(""); // the string to print
    let mut print = true; // whether the string should be printed

    println!(
        "  0x{:03x}:                               | .pos 0x{:03x} rodata",
        phdr.vaddr, phdr.vaddr
    );

    // loop through memory
    let mut i: i32 = 0;
    let mut mem = addr;
    while mem < max_addr {
        if i == 0 {
            // 0 represents the beginning of a row
            print!("  0x{:03x}: ", addr);

            // s will be printed at the end of this row, so get s
            s = String::from("");
            let mut mem2 = mem;

            let mut byte = memory.get(mem2 as usize).unwrap();
            while *byte != 0 {
                byte = match memory.get(mem2 as usize) {
                    Some(v) => v,
                    None => break,
                };
                s.push(*byte as char);
                mem2 += 1;
            }
        }

        let byte = memory.get(mem as usize).unwrap();
        print!("{:02x} ", byte);

        // end of row
        if i == 9 || *byte == 0 {
            // pad
            while i < 9 {
                print!("   ");
                i += 1;
            }

            print!("| ");

            if print {
                print!("  .string \"{}\"", s);
                print = false;
            }

            if *byte == 0 {
                print = true;
            }

            println!();
            i = -1; // -1 because i = 0 at next loop
        }

        i += 1;
        mem += 1;
        addr += 1;
    }
}

fn inv_mem(inst: &mut Inst, cpu: &mut Cpu) {
    inst.icode = Icode::INVALID;
    cpu.stat = Stat::ADR;
}

fn inv_inst(inst: &mut Inst, cpu: &mut Cpu) {
    inst.icode = Icode::INVALID;
    cpu.stat = Stat::INS;
}

fn memtoi(memory: &Box<[u8]>, start: Address) -> Option<Address> {
    let mut n: Address = 0;

    for i in (start..(start + 8)).rev() {
        let byte = match memory.get(i as usize) {
            Some(v) => v,
            None => return None,
        };

        n <<= 8;
        n += *byte as Address;
    }

    Some(n)
}
