use std::{
    fmt::Display,
    str::{from_utf8, FromStr},
};

use crate::Error;

fn are_bytes_uppercase_lowercase_chars(bytes: [u8; 4]) -> bool {
    bytes
        .iter()
        .all(|&b| b.is_ascii_uppercase() || b.is_ascii_lowercase())
}

#[derive(PartialEq, Debug)]
pub struct ChunkType {
    bytes: [u8; 4],
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    fn try_from(bytes: [u8; 4]) -> Result<Self, Error> {
        if !are_bytes_uppercase_lowercase_chars(bytes) {
            println!("{:?}", bytes);
            return Err("Bytes must be uppercase or lowercase letters".into());
        }

        Ok(Self { bytes })
    }
}

impl FromStr for ChunkType {
    type Err = Error;

    fn from_str(str: &str) -> Result<Self, Error> {
        let chunk_bytes: [u8; 4] = str
            .as_bytes()
            .try_into()
            .map_err(|_| "String must have a 4-byte length")?;

        let chunk_type = Self { bytes: chunk_bytes };

        if !are_bytes_uppercase_lowercase_chars(chunk_bytes) {
            return Err("Bytes must be uppercase or lowercase letters".into());
        }

        Ok(chunk_type)
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Using unwrap because our bytes are already limited to lowercase characters
        write!(f, "{}", from_utf8(&self.bytes).unwrap())
    }
}

/// Defined in http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html
impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    fn get_ancillary_bit(&self) -> u8 {
        let first_byte = &self.bytes[0];
        (*first_byte >> 5) & 1
    }

    fn get_private_bit(&self) -> u8 {
        let first_byte = &self.bytes[1];
        (*first_byte >> 5) & 1
    }

    fn get_reserved_bit(&self) -> u8 {
        let first_byte = &self.bytes[2];
        (*first_byte >> 5) & 1
    }

    fn get_safe_to_copy_bit(&self) -> u8 {
        let first_byte = &self.bytes[3];
        (*first_byte >> 5) & 1
    }

    fn is_critical(&self) -> bool {
        self.get_ancillary_bit() == 0
    }

    fn is_public(&self) -> bool {
        self.get_private_bit() == 0
    }

    fn is_reserved_bit_valid(&self) -> bool {
        self.get_reserved_bit() == 0
    }

    fn is_safe_to_copy(&self) -> bool {
        self.get_safe_to_copy_bit() == 1
    }

    fn is_valid(&self) -> bool {
        if self.get_reserved_bit() != 0 {
            return false;
        }

        are_bytes_uppercase_lowercase_chars(self.bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
