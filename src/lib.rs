use core::error;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};

#[derive(PartialEq, Debug)]
pub enum Error {
    SmallSize,
    Io(io::ErrorKind),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::SmallSize => write!(f, "file size is less than 64 KB"),
            Self::Io(err_kind) => write!(f, "{}", err_kind),
        }
    }
}

impl error::Error for Error {}

#[derive(PartialEq, Debug)]
pub struct MovieHash(pub u64);

const CHUNK_SIZE: u64 = 65536;

impl MovieHash {
    pub fn new(hash: u64) -> Self {
        MovieHash(hash)
    }

    pub fn as_hex(&self) -> String {
        format!("{:016x}", self.0)
    }

    pub fn from_path(path: &str) -> Result<Self, Error> {
        let file = File::open(path).map_err(Error::from)?;
        let file_size = file.metadata().map_err(Error::from)?.len();

        if file_size < CHUNK_SIZE {
            return Err(Error::SmallSize);
        };

        let mut hash: u64 = file_size;
        let mut reader = BufReader::with_capacity(CHUNK_SIZE as usize, file);
        let mut word_buffer = [0u8; 8];
        let word_count = CHUNK_SIZE / 8;

        for _ in 0..word_count {
            reader.read_exact(&mut word_buffer).map_err(Error::from)?;
            hash = hash.wrapping_add(u64::from_le_bytes(word_buffer));
        }

        reader
            .seek(SeekFrom::Start(file_size - CHUNK_SIZE))
            .map_err(Error::from)?;

        for _ in 0..word_count {
            reader.read_exact(&mut word_buffer).map_err(Error::from)?;
            hash = hash.wrapping_add(u64::from_le_bytes(word_buffer));
        }

        Ok(MovieHash::new(hash))
    }
}

impl Display for MovieHash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_not_found_error() {
        assert_eq!(
            MovieHash::from_path("test-files/non-existing.mp4"),
            Err(Error::Io(io::ErrorKind::NotFound))
        );
    }

    #[test]
    fn should_return_small_size_error() {
        assert_eq!(
            MovieHash::from_path("test-files/small.txt"),
            Err(Error::SmallSize)
        );
    }

    #[test]
    fn should_return_valid_hash() {
        assert_eq!(
            MovieHash::from_path("test-files/breakdance.avi")
                .unwrap()
                .as_hex(),
            "8e245d9679d31e12"
        );
    }

    #[test]
    fn should_print_human_readable_error_messages() {
        let test_cases = [
            (Error::SmallSize, "file size is less than 64 KB"),
            (Error::Io(io::ErrorKind::NotFound), "entity not found"),
            (
                Error::Io(io::ErrorKind::InvalidFilename),
                "invalid filename",
            ),
        ];

        for (err, message) in test_cases {
            assert_eq!(format!("{}", err), message)
        }
    }
}
