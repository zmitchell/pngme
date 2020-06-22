use crate::chunk_type::{ChunkType, ChunkTypeError};
use crc::crc32::checksum_ieee;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ChunkError {
    UTF8Error(std::string::FromUtf8Error),
    ChunkTypeError(ChunkTypeError),
    InvalidCRC(u32, u32),
    LengthMismatch(u32, u32),
    ChunkTooShort,
}

impl fmt::Display for ChunkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChunkError::UTF8Error(err) => write!(
                f,
                "Data is not valid UTF-8 and cannot be converted into a string.\n{}",
                err
            ),
            ChunkError::ChunkTypeError(err) => write!(f, "Invalid chunk type: {}", err),
            ChunkError::InvalidCRC(found, expected) => {
                write!(f, "Invalid CRC, found {}, expected {}", found, expected)
            }
            ChunkError::LengthMismatch(found, expected) => write!(
                f,
                "Invalid data length, found {} bytes, expected {} bytes",
                found, expected
            ),
            ChunkError::ChunkTooShort => write!(f, "Chunk must be at least 12 bytes long."),
        }
    }
}

impl Error for ChunkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ChunkError::UTF8Error(err) => Some(err),
            ChunkError::ChunkTypeError(err) => Some(err),
            _ => None,
        }
    }
}

impl std::convert::From<ChunkTypeError> for ChunkError {
    fn from(err: ChunkTypeError) -> ChunkError {
        ChunkError::ChunkTypeError(err)
    }
}

#[derive(Debug)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let total_length = bytes.len();
        if total_length < 12 {
            return Err(ChunkError::ChunkTooShort);
        }
        let mut length_bytes = [0; 4];
        bytes[0..4]
            .iter()
            .enumerate()
            .for_each(|(i, x)| length_bytes[i] = *x);
        let length = u32::from_be_bytes(length_bytes);
        let mut ct_bytes = [0; 4];
        bytes[4..8]
            .iter()
            .enumerate()
            .for_each(|(i, x)| ct_bytes[i] = *x);
        let chunk_type = ChunkType::try_from(ct_bytes)?;
        let mut crc_bytes = [0; 4];
        bytes[(total_length - 4)..]
            .iter()
            .enumerate()
            .for_each(|(i, x)| crc_bytes[i] = *x);
        let crc = u32::from_be_bytes(crc_bytes);
        let mut data = Vec::new();
        data.extend_from_slice(&bytes[8..(total_length - 4)]);
        let bytes_for_crc = chunk_type
            .bytes
            .iter()
            .chain(data.as_slice().iter())
            .copied()
            .collect::<Vec<u8>>();
        let computed_crc = checksum_ieee(bytes_for_crc.as_slice());
        if computed_crc != crc {
            return Err(ChunkError::InvalidCRC(computed_crc, crc));
        }
        Ok(Chunk {
            length,
            chunk_type,
            data,
            crc,
        })
    }
}

impl Chunk {
    fn new(chunk_type: ChunkType, data: Vec<u8>) -> Result<Self, std::num::TryFromIntError> {
        let length = u32::try_from(data.len())?;
        let bytes_for_crc = chunk_type
            .bytes
            .iter()
            .chain(data.as_slice().iter())
            .copied()
            .collect::<Vec<u8>>();
        let crc = checksum_ieee(&bytes_for_crc);
        Ok(Chunk {
            length,
            chunk_type,
            data,
            crc,
        })
    }

    fn length(&self) -> u32 {
        self.length
    }

    fn chunk_type(&self) -> ChunkType {
        self.chunk_type
    }

    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn crc(&self) -> u32 {
        self.crc
    }

    fn data_as_string(&self) -> Result<String, ChunkError> {
        std::string::String::from_utf8(self.data.clone()).map_err(|err| ChunkError::UTF8Error(err))
    }

    fn as_bytes(&self) -> Vec<u8> {
        let bytes: Vec<u8> = self
            .length
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type.bytes.iter())
            .chain(self.data.iter())
            .chain(self.crc.to_be_bytes().iter())
            .copied()
            .collect();
        return bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data: Vec<u8> = "This is where your secret message will be!"
            .bytes()
            .collect();
        Chunk::new(chunk_type, data).unwrap()
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
}
