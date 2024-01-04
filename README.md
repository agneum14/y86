# Y86 Disassembler and Simulator

This utility disassembles Y86 object files compiled in the Mini-ELF format 
and simulates the execution by virtualizing the Y86 CPU and memory. Since this 
Mini-ELF format is scarcely known, another utility for assembling these object 
files from Y86 source code may be forthcoming, depending on my motivation. This 
is a rewrite of a project I did in my systems programming class, ported from C 
to Rust to avoid violating the Honor Code.

## Usage

```
Usage: y86sim [OPTIONS] <FILE>

Arguments:
  <FILE>  Mini-ELF object file

Options:
  -H             Show the Mini-ELF header
  -a             Show all with brief memory
  -f             Show all with full memory
  -s             Show the program headers
  -m             Show the memory contents (brief)
  -M             Show the memory contents (full)
  -d             Disassemble code contents
  -D             Disassemble data contents
  -e             Execute program
  -E             Execute program (trace mode)
  -h, --help     Print help
  -V, --version  Print version
```
