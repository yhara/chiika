# chiika

Exploration for LLVM + tokio

## chiika-1

- A language that compiles to LLVM IR
- All functions returns a value (No `void`. Use `0` for `void`)

## chiika_runtime

- Runtime written in Rust
- Built as staticlib and linked with the chiika-1 program

## chiika-2

- A language that compiles to chiika-1
- Has notion of asyncness
  - Async externs are declared with `extern_async`.

## Prerequisites

- Rust (tested with 1.74.1)
- LLVM 16

You also may want to install Ruby (see Rakefile)

## How to run

see Rakefile

## Restriction 

- 64-bit OS only (assumes pointer size is 64bits)

## License

MIT
