//! A binary parsing utility that provides a cursor-like interface for reading binary data.
//! 
//! This module provides a `BinaryCursor` type that wraps a `std::io::Cursor<T>` where T implements `AsRef<[u8]>`
//! and adds functionality for parsing various types of binary data, as well as managing a location stack
//! for temporary position changes.
//! 
//! This project is heavily inspired by nom, but with the intention of not consuming the input data.
//! 
//! # Examples
//! 
//! ```rust
//! use cursor_binary_parser::binary_cursor::{BinaryCursor, BinaryCursorJump};
//! 
//! // Can be used with Vec<u8>
//! let data = vec![0x42, 0x24, 0x00, 0x01];
//! let mut cursor = BinaryCursor::new(&data);
//! 
//! // Parse a u8
//! let value = cursor.parse_u8().unwrap();
//! assert_eq!(value, 0x42);
//! 
//! // Can also be used with &[u8]
//! let slice: &[u8] = &[0x42, 0x24, 0x00, 0x01];
//! let mut cursor = BinaryCursor::new(slice);
//! 
//! // Use BinaryCursorJump for temporary position changes
//! {
//!     let mut jump = BinaryCursorJump::new(&mut cursor);
//!     jump.jump(2).unwrap();
//!     let value = jump.cursor.parse_u16_le().unwrap();
//!     assert_eq!(value, 0x0100);
//! }
//! // Position is automatically restored after jump
//! ```

pub mod binary_cursor;
