# cursor_binary_parser
[![Rust](https://github.com/GrumpyMetalGuy/cursor_binary_parser/actions/workflows/rust.yml/badge.svg)](https://github.com/GrumpyMetalGuy/cursor_binary_parser/actions/workflows/rust.yml) [![Crates.io Version](https://img.shields.io/crates/v/cursor_binary_parser)](https://crates.io/crates/cursor_binary_parser) [![docs.rs](https://img.shields.io/docsrs/cursor_binary_parser)](https://docs.rs/cursor_binary_parser)


A [nom](<https://github.com/rust-bakery/nom>)-like wrapper around a Cursor of u8 to provide non-consuming parsing features.

## Motivation
Needing to parse a binary file structure with internal offsets into different structures, I investigated [nom](<https://github.com/rust-bakery/nom>), but found that due to the way it consumes input data, all offsets became invalid. The Cursor Binary Parser provides a ```nom```-like interface to allow binary structures to be parsed without consuming the input stream.

Direct string creation is not directly supported yet.

## Usage

```rust
use cursor_binary_parser::binary_cursor::{BinaryCursor, BinaryCursorJump};
use std::io;

fn main() -> Result<(), io::Error> {
    // Example 1: Using Vec<u8>
    let vec_data = vec![
        0x42, // u8 = 66
        0x24, 0x00, // u16_le = 36
        0x01, 0x43, 0x25, 0x01, // u32_le = 19,219,201
        0x00, 0x00, 0x80, 0x3F, // f32_le = 1.0
        0x01, 0x02, 0x03, 0x04, // bytes = [1, 2, 3, 4]
    ];

    println!("Example 1: Using Vec<u8>");
    let mut cursor = BinaryCursor::new(&vec_data);
    let u8_val = cursor.parse_u8().unwrap();
    println!("First u8: 0x{:02X} ({})", u8_val, u8_val);

    // Example 2: Using &[u8]
    let slice_data: &[u8] = &[
        0x42, 0x24, 0x00, 0x01, 0x43, 0x25, 0x01, 0x02,
        0x00, 0x00, 0x80, 0x3F, 0x01, 0x02, 0x03, 0x04,
    ];

    println!("\nExample 2: Using &[u8]");
    let mut cursor = BinaryCursor::new(slice_data);
    let u8_val = cursor.parse_u8().unwrap();
    println!("First u8: 0x{:02X} ({})", u8_val, u8_val);

    // Example 3: Using BinaryCursorJump with Vec<u8>
    println!("\nExample 3: Using BinaryCursorJump");
    let mut cursor = BinaryCursor::new(&vec_data);
    {
        let mut jump = BinaryCursorJump::new(&mut cursor);
        jump.jump(3).unwrap();
        let u32_val = jump.cursor.parse_u32_le().unwrap();
        println!("u32 at position 3: 0x{:08X} ({})", u32_val, u32_val);
    }
    // Position is automatically restored
    println!("Position after jump: {}", cursor.position());

    // Read data after jump to demonstrate position restoration
    println!("\nReading data after jump:");
    let u8_val = cursor.parse_u8().unwrap();
    println!("u8 after jump: 0x{:02X} ({})", u8_val, u8_val);
    let u16_val = cursor.parse_u16_le().unwrap();
    println!("u16 after jump: 0x{:04X} ({})", u16_val, u16_val);
    let u32_val = cursor.parse_u32_le().unwrap();
    println!("u32 after jump: 0x{:04X} ({})", u32_val, u32_val);
    let f32_val = cursor.parse_f32_le().unwrap();
    println!("f32 after jump: {} (0x{:08X})", f32_val, f32_val.to_bits());
    let bytes = cursor.parse_bytes(4).unwrap();
    println!("bytes after jump: {:02X?} ({:?})", bytes, bytes);

    Ok(())
} 
```

## Contributions
Although this is a fairly simple library, if you can think of anything that could be done to improve it, please open an issue or submit a PR!

## License
This project is licensed under

 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)
