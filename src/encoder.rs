pub trait AsBencodeBytes {
    fn as_bytes(&self) -> &Vec<u8>;
}

