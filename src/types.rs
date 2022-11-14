use std::fmt;
use std::str::Utf8Error;
use std::str::from_utf8;

#[derive(Debug, PartialEq)]
pub enum BencodeError {
    FileRead(String),
    UnrecognizedByte(String),
    UnexpectedEndMarker,
    BytestreamEnded,
    IntParseAscii(Utf8Error),
    IntParseInt(String),
    IntParseLeadingZero,
    IntParseNegativeZero,
    StrParseLeadingZero,
    StrLenInvalidByte,
    StrParse,
    DictKeyParse
}

#[derive(Debug, PartialEq)]
pub struct ByteString {
    pub bytes: Vec<u8>
}

impl ByteString {
    pub fn new(bytes: Vec<u8>) -> Self {
        ByteString { bytes: bytes }
    }
}

#[derive(Debug, PartialEq)]
pub enum BencodeItem {
    String(ByteString),
    Int(i64),
    List(Vec<BencodeItem>),
    Dict(Vec<(String, BencodeItem)>)
}

impl fmt::Display for BencodeItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BencodeItem::String(s) => {
                if let Ok(s) = String::try_from(s) {
                    write!(f, "\"{}\"", s)
                } else {
                    write!(f, "Bytes(len={})", s.bytes.len())
                }
            },
            BencodeItem::Int(i) => write!(f, "{}", i),
            BencodeItem::List(l) => {
                write!(f, "[")?;
                for item in l {
                    write!(f, "{},", item)?;
                }
                write!(f, "]")
            },
            BencodeItem::Dict(d) => {
                write!(f, "{{\n")?;
                for (key, value) in &*d {
                    write!(f, " \"{}\": {},\n", key, value)?;
                }
                write!(f, "\n}}")
            }
        }
    }
}
