use std::fs;
use std::path::Path;

pub use types::BencodeError;
pub use types::BencodeItem;
pub use types::ByteString;
pub use encoder::AsBencodeBytes;

mod c;
mod types;
mod decoder;
mod encoder;

pub fn open<P>(path: P) -> Result<BencodeItem, BencodeError> where P: AsRef<Path> + std::fmt::Display {
    let res = &fs::read(&path);
    match res {
        Err(e) => Err(
            BencodeError::FileRead(format!("couldn't read path {}: {}", path, e))
        ),
        Ok(b) => decoder::parse_bytes(&mut b.iter().peekable()),
    }
}
