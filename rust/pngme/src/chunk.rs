use crate::{chunk_type::ChunkType, Error};
use crc::Crc;
use std::{fmt::Display, str};

const CRC_INSTANCE: crc::Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

pub struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
    length: usize,
    crc: u32,
}

impl TryFrom<&Vec<u8>> for Chunk {
    type Error = Error;

    fn try_from(input: &Vec<u8>) -> Result<Self, Self::Error> {
        // First 4 bytes
        let length_bytes: &[u8; 4] = &input[0..4]
            .try_into()
            .expect("Length can't be converted to number from bytes");

        let length = u32::from_be_bytes(*length_bytes) as usize;

        // Next 4 bytes
        let chunk_type_bytes: &[u8; 4] = &input[4..8]
            .try_into()
            .expect("Could not convert chunk_type to 4-byte array");

        let chunk_type: ChunkType = ChunkType::try_from(*chunk_type_bytes)?;

        // 4 because that's the size of the CRC in bytes
        let crc_index = input.len() - 4;

        let crc_bytes = &input[crc_index..];
        let crc = u32::from_be_bytes(
            crc_bytes
                .try_into()
                .expect("CRC can't be converted to number from bytes"),
        );

        let data: Vec<u8> = input[8..crc_index].to_vec();

        let mut crc_target = chunk_type.bytes().to_vec();
        crc_target.extend_from_slice(&data);

        let calculated_crc = CRC_INSTANCE.checksum(&crc_target);

        if crc != calculated_crc {
            return Err("The provided CRC does not match the expected one".into());
        }

        Ok(Self {
            length,
            chunk_type,
            data,
            crc,
        })
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Chunk Type = {}. Data = {}. Length = {}. CRC = {}",
            self.chunk_type.to_string(),
            self.data_as_string()
                .expect("Data cannot be converted to String"),
            self.length,
            self.crc
        )
    }
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let mut crc_target = chunk_type.bytes().to_vec();
        crc_target.extend_from_slice(&data);

        let crc = CRC_INSTANCE.checksum(&crc_target);

        Self {
            length: data.len(),
            chunk_type,
            data,
            crc,
        }
    }

    fn crc(&self) -> u32 {
        self.crc
    }

    fn length(&self) -> usize {
        self.length
    }

    fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    fn data_as_string(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(&self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
