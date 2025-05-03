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

use std::io::{Cursor, Read};
use thiserror::Error;

// region: Error implementation
/// Error type for binary cursor operations
#[derive(Debug, Error)]
pub enum BinaryCursorError {
    /// Error that occurs during parsing operations
    #[error("Parse error: {0}")]
    ParseError(#[from] std::io::Error),
}

impl BinaryCursorError {
    /// Creates a new `BinaryCursorError` from an `io::Error`
    pub fn from_io_error(error: std::io::Error) -> Self {
        Self::ParseError(error)
    }
}
// endregion: Error implementation

// region: Cursor implementation
/// A cursor-like interface for parsing binary data
/// 
/// This type provides methods for parsing various types of binary data and managing
/// a location stack for temporary position changes. It works with any type T that
/// implements `AsRef<[u8]>`, such as `Vec<u8>`, `&[u8]`, or other byte containers.
#[derive(Debug)]
pub struct BinaryCursor<T: AsRef<[u8]>> {
    /// The underlying cursor containing the binary data
    pub data: Cursor<T>,
    /// Stack of saved positions for temporary jumps
    location_stack: Vec<u32>,
}

impl<T> BinaryCursor<T>
where
    T: AsRef<[u8]>,
{
    /// Creates a new `BinaryCursor` from a slice of bytes
    pub fn new(data: T) -> Self {
        Self {
            data: Cursor::new(data),
            location_stack: vec![],
        }
    }

    /// Saves the current position to the location stack
    pub fn push_location(&mut self) {
        let pos = self.data.position() as u32;
        self.location_stack.push(pos);
    }

    /// Removes and returns the most recently saved position from the location stack
    pub fn pop_location(&mut self) -> Option<u32> {
        self.location_stack.pop()
    }

    /// Restores the most recently saved position from the location stack
    /// 
    /// Returns `true` if a position was restored, `false` if the stack was empty
    pub fn restore_location(&mut self) -> bool {
        if let Some(pos) = self.location_stack.pop() {
            self.data.set_position(pos as u64);
            true
        } else {
            false
        }
    }

    /// Parses a single u8 from the current position
    pub fn parse_u8(&mut self) -> Result<u8, BinaryCursorError> {
        let mut buf = [0u8; 1];
        self.data.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    /// Parses a u16 in little-endian format from the current position
    pub fn parse_u16_le(&mut self) -> Result<u16, BinaryCursorError> {
        let mut buf = [0u8; 2];
        self.data.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    /// Parses a u32 in little-endian format from the current position
    pub fn parse_u32_le(&mut self) -> Result<u32, BinaryCursorError> {
        let mut buf = [0u8; 4];
        self.data.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    /// Parses an f32 in little-endian format from the current position
    pub fn parse_f32_le(&mut self) -> Result<f32, BinaryCursorError> {
        let mut buf = [0u8; 4];
        self.data.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    /// Parses a specified number of bytes from the current position
    pub fn parse_bytes(&mut self, count: usize) -> Result<Vec<u8>, BinaryCursorError> {
        let mut buf = vec![0u8; count];
        self.data.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Returns the current position in the data stream
    pub fn position(&self) -> u64 {
        self.data.position()
    }

    /// Sets the current position in the data stream
    pub fn set_position(&mut self, pos: u64) {
        self.data.set_position(pos);
    }

    /// Parses multiple items using the provided parser function
    /// 
    /// This is similar to nom's `count` combinator, but works with the `BinaryCursor` interface.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use cursor_binary_parser::binary_cursor::BinaryCursor;
    /// 
    /// let data = vec![0x01, 0x02, 0x03, 0x04];
    /// let mut cursor = BinaryCursor::new(data);
    /// 
    /// let values = cursor.count(|c| c.parse_u8(), 4).unwrap();
    /// assert_eq!(values, vec![0x01, 0x02, 0x03, 0x04]);
    /// ```
    pub fn count<U, F>(&mut self, mut parser: F, count: usize) -> Result<Vec<U>, BinaryCursorError>
    where
        F: FnMut(&mut Self) -> Result<U, BinaryCursorError>,
    {
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(parser(self)?);
        }
        Ok(items)
    }
}
// endregion: Cursor implementation

// region: CursorJump implementation
/// A helper type for temporary position changes
/// 
/// This type provides a way to temporarily change the position of a `BinaryCursor`
/// and automatically restore it when the `BinaryCursorJump` is dropped.
/// Works with any type T that implements `AsRef<[u8]>`.
pub struct BinaryCursorJump<'a, T: AsRef<[u8]>> {
    /// Reference to the cursor being manipulated
    pub cursor: &'a mut BinaryCursor<T>,
}

impl<'a, T> BinaryCursorJump<'a, T> 
where
    T: AsRef<[u8]>,
{
    /// Creates a new `BinaryCursorJump` for the given cursor
    pub fn new(cursor: &'a mut BinaryCursor<T>) -> Self {
        Self { cursor }
    }

    /// Temporarily jumps to the specified position
    /// 
    /// The position will be automatically restored when the `BinaryCursorJump` is dropped.
    pub fn jump(&mut self, location: u64) -> Result<(), BinaryCursorError> {
        self.cursor.push_location();
        self.cursor.set_position(location);
        Ok(())
    }
}

impl<'a, T> Drop for BinaryCursorJump<'a, T>
where
    T: AsRef<[u8]>,
{
    fn drop(&mut self) {
        self.cursor.restore_location();
    }
}
// endregion: CursorJump implementation

// region: Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_u8() {
        let data = vec![0x42, 0x43, 0x44];
        let mut cursor = BinaryCursor::new(data);
        assert_eq!(cursor.parse_u8().unwrap(), 0x42);
        assert_eq!(cursor.position(), 1);
    }

    #[test]
    fn test_parse_u16_le() {
        let data = vec![0x42, 0x24, 0x43, 0x25];
        let mut cursor = BinaryCursor::new(data);
        assert_eq!(cursor.parse_u16_le().unwrap(), 0x2442);
        assert_eq!(cursor.position(), 2);
    }

    #[test]
    fn test_parse_u32_le() {
        let data = vec![0x42, 0x24, 0x00, 0x01, 0x43, 0x25, 0x01, 0x02];
        let mut cursor = BinaryCursor::new(data);
        assert_eq!(cursor.parse_u32_le().unwrap(), 0x01002442);
        assert_eq!(cursor.position(), 4);
    }

    #[test]
    fn test_parse_f32_le() {
        let data = vec![0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x40];
        let mut cursor = BinaryCursor::new(data);
        assert_eq!(cursor.parse_f32_le().unwrap(), 1.0);
        assert_eq!(cursor.position(), 4);
    }

    #[test]
    fn test_parse_bytes() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut cursor = BinaryCursor::new(data);
        assert_eq!(cursor.parse_bytes(4).unwrap(), vec![0x01, 0x02, 0x03, 0x04]);
        assert_eq!(cursor.position(), 4);
    }

    #[test]
    fn test_location_stack() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut cursor = BinaryCursor::new(data);

        cursor.push_location();
        cursor.set_position(4);
        assert_eq!(cursor.position(), 4);

        assert!(cursor.restore_location());
        assert_eq!(cursor.position(), 0);
    }

    #[test]
    fn test_binary_cursor_jump() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut cursor = BinaryCursor::new(data);

        {
            let mut jump = BinaryCursorJump::new(&mut cursor);
            jump.jump(4).unwrap();
            assert_eq!(jump.cursor.position(), 4);
        }

        assert_eq!(cursor.position(), 0u64);
    }

    #[test]
    fn test_sequential_parsing() {
        let data = vec![0x42, 0x24, 0x00, 0x01, 0x43, 0x25, 0x01, 0x02];
        let mut cursor = BinaryCursor::new(data);

        assert_eq!(cursor.parse_u8().unwrap(), 0x42);
        assert_eq!(cursor.position(), 1);

        assert_eq!(cursor.parse_u16_le().unwrap(), 0x0024);
        assert_eq!(cursor.position(), 3);

        assert_eq!(cursor.parse_u32_le().unwrap(), 0x01254301);
        assert_eq!(cursor.position(), 7);

        assert!(cursor.parse_u8().is_ok());
        assert_eq!(cursor.position(), 8);
    }

    #[test]
    fn test_count() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut cursor = BinaryCursor::new(data);
        let result = cursor.count(|c| c.parse_u8(), 4).unwrap();
        assert_eq!(result, vec![0x01, 0x02, 0x03, 0x04]);
        assert_eq!(cursor.position(), 4);
    }

    #[test]
    fn test_pop_location() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut cursor = BinaryCursor::new(data);
        assert_eq!(cursor.pop_location(), None);

        cursor.push_location();
        cursor.set_position(2);
        assert_eq!(cursor.pop_location(), Some(0));
        assert_eq!(cursor.position(), 2);
    }

    #[test]
    fn test_error_handling() {
        let data = vec![0x42];
        let mut cursor = BinaryCursor::new(data);

        assert!(cursor.parse_u16_le().is_err());
        assert!(cursor.parse_u32_le().is_err());
        assert!(cursor.parse_f32_le().is_err());
        assert!(cursor.parse_bytes(2).is_err());
    }

    #[test]
    fn test_restore_location() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut cursor = BinaryCursor::new(data);

        cursor.push_location();
        cursor.set_position(2);
        assert_eq!(cursor.position(), 2);

        assert!(cursor.restore_location());
        assert_eq!(cursor.position(), 0);

        assert!(!cursor.restore_location());
    }

    #[test]
    fn test_error_conversion() {
        use std::io::{Error, ErrorKind};
        let io_error = Error::new(ErrorKind::UnexpectedEof, "test error");
        let cursor_error = BinaryCursorError::from_io_error(io_error);
        match cursor_error {
            BinaryCursorError::ParseError(_) => (),
        }
    }
}
// endregion: Tests
