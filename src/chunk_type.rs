use std::convert::TryFrom;
use std::fmt;

#[derive(Debug)]
pub enum ChunkTypeError {
    InvalidByte(u8),
    InvalidLength,
}

impl fmt::Display for ChunkTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChunkTypeError::InvalidByte(byte) => write!(f, "Invalid byte: {}", byte),
            ChunkTypeError::InvalidLength => {
                write!(f, "Chunk types must be 4 characters (bytes) long")
            }
        }
    }
}

impl std::error::Error for ChunkTypeError {}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct ChunkType {
    pub bytes: [u8; 4],
}

impl ChunkType {
    fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    /// Returns true if the chunk type is valid.
    ///
    /// Valid chunk types follow these rules:
    /// 1) They must consist of ASCII letters (uppercase or lowercase).
    /// 2) The third character must be uppercase.
    fn is_valid(&self) -> bool {
        // Lowercase ASCII
        let lower = 65u8..=90;
        // Uppercase ASCII
        let upper = 97u8..=122;
        for byte in self.bytes.iter() {
            if !(lower.contains(&byte)) && !(upper.contains(&byte)) {
                return false;
            }
        }
        // Third character is uppercase
        if (self.bytes[2] & 32) == 32 {
            return false;
        }
        return true;
    }

    /// Returns true if bit 5 of the first byte is 0.
    ///
    /// A critical chunk type is necessary to meaningfully display the contents of the file.
    fn is_critical(&self) -> bool {
        return (self.bytes[0] & 32) == 0;
    }

    fn is_public(&self) -> bool {
        return (self.bytes[1] & 32) == 0;
    }

    fn is_reserved_bit_valid(&self) -> bool {
        return (self.bytes[2] & 32) == 0;
    }

    fn is_safe_to_copy(&self) -> bool {
        return (self.bytes[3] & 32) == 32;
    }
}

impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            String::from(std::str::from_utf8(&self.bytes).unwrap())
        )
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = ChunkTypeError;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        let lower = 65u8..=90;
        let upper = 97u8..=122;
        for byte in value.iter() {
            if !(lower.contains(&byte)) && !(upper.contains(&byte)) {
                return Err(ChunkTypeError::InvalidByte(*byte));
            }
        }
        Ok(ChunkType { bytes: value })
    }
}

impl std::str::FromStr for ChunkType {
    type Err = ChunkTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(ChunkTypeError::InvalidLength);
        }
        let sb = s.as_bytes();
        let bytes = [sb[0], sb[1], sb[2], sb[3]];
        ChunkType::try_from(bytes)
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
}
