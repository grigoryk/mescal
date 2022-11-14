use core::slice::Iter;
use std::iter::Peekable;
use std::str::from_utf8;

use crate::{BencodeItem, BencodeError, ByteString};
use crate::c;

pub fn parse_bytes(bytes_iter: &mut Peekable<Iter<u8>>) -> Result<BencodeItem, BencodeError> {
    match bytes_iter.peek() {
        Some(&&b) => match b {
            c::M_DICT => Ok(BencodeItem::Dict(read_dict(bytes_iter)?)),
            c::M_INT => Ok(BencodeItem::Int(read_int(bytes_iter)?)),
            c::M_LIST => Ok(BencodeItem::List(read_list(bytes_iter)?)),
            c::M_0..=c::M_9 => Ok(BencodeItem::String(read_string(bytes_iter)?)),
            c::M_END => Err(BencodeError::UnexpectedEndMarker),
            _ => Err(
                BencodeError::UnrecognizedByte(format!("unrecognized byte: {}", b))
            )
        },
        None => Err(BencodeError::BytestreamEnded)
    }
}

fn read_dict(bytes_iter: &mut Peekable<Iter<u8>>) -> Result<Vec<(String, BencodeItem)>, BencodeError> {
    // consume 'd'
    bytes_iter.next();
    let mut res: Vec<(String, BencodeItem)> = vec!();
    // empty dict
    if let Some(&&c::M_END) = bytes_iter.peek() {
        return Ok(res)
    }
    loop {
        if let Ok(key) = String::try_from(&read_string(bytes_iter)?) {
            res.push((key, parse_bytes(bytes_iter)?));
        } else {
            return Err(BencodeError::DictKeyParse)
        }

        if let Some(&&c::M_END) = bytes_iter.peek() {
            bytes_iter.next();
            break;
        }
    }
    Ok(res)
}

fn read_list(mut bytes_iter: &mut Peekable<Iter<u8>>) -> Result<Vec<BencodeItem>, BencodeError> {
    // consume 'l'
    bytes_iter.next();

    let mut res: Vec<BencodeItem> = vec!();
    loop {
        match bytes_iter.peek() {
            // empty list
            Some(&&c::M_END) => {
                bytes_iter.next(); // consume 'e'
                break;
            },
            Some(_) => {
                res.push(parse_bytes(&mut bytes_iter)?);
            },
            None => return Err(BencodeError::BytestreamEnded),
        }
    }
    Ok(res)
}

fn read_int(bytes_iter: &mut Peekable<Iter<u8>>) -> Result<i64, BencodeError> {
    let mut buff: Vec<u8> = vec!();
    let mut b: &u8;

    // consume 'i'
    bytes_iter.next();

    loop {
        let curr_byte = bytes_iter.next();

        if curr_byte.is_none() {
            return Err(BencodeError::BytestreamEnded)
        }
        b = curr_byte.unwrap();
        if buff.len() == 0 && *b == c::M_END {
            return Err(BencodeError::UnexpectedEndMarker)
        } else if *b == c::M_END {
            break;
        }
        // -0 not allowed
        if *b == c::M_DASH {
            if let Some(&&c::M_0) = bytes_iter.peek() {
                return Err(BencodeError::IntParseNegativeZero)
            }
        }
        // leading zeros not allowed
        if buff.len() == 0 && *b == c::M_0 {
            if let Some(&&c::M_END) = bytes_iter.peek() {} else {
                return Err(BencodeError::IntParseLeadingZero)
            }
        }
        buff.push(*b);
    }

    let res = ascii_bytes_to_int(&buff);
    res
}

fn ascii_bytes_to_int(bytes: &Vec<u8>) -> Result<i64, BencodeError> {
    match from_utf8(&bytes) {
        Ok(s) => match s.parse::<i64>() {
            Ok(i) => Ok(i),
            Err(e) => Err(BencodeError::IntParseInt(format!("{}", e))),
        },
        Err(e) => Err(BencodeError::IntParseAscii(e))
    }
}

impl TryFrom<&ByteString> for String {
    type Error = ();

    fn try_from(value: &ByteString) -> Result<Self, Self::Error> {
        match from_utf8(&value.bytes) {
            Ok(s) => Ok(String::from(s)),
            Err(_) => Err(())
        }
    }
}

fn read_string(bytes_iter: &mut Peekable<Iter<u8>>) -> Result<ByteString, BencodeError> {
    let mut len_buff = vec!();
    loop {
        let b = bytes_iter.next();
        match b {
            Some(&c::M_COLON) => break,
            Some(c::M_0..=c::M_9) => {
                // empty string handling
                if len_buff.len() == 0 {
                    if *b.unwrap() == c::M_0 {
                        if let Some(&&c::M_COLON) = bytes_iter.peek() {
                            bytes_iter.next(); // consume the colon
                            return Ok(ByteString::new(vec!()));
                        } else {
                            return Err(BencodeError::StrParseLeadingZero);
                        }
                    }
                }
                len_buff.push(*b.unwrap())
            },
            Some(_) => return Err(BencodeError::StrLenInvalidByte),
            None => return Err(BencodeError::BytestreamEnded),
        }
    }
    let str_len = ascii_bytes_to_int(&len_buff)?;
    let mut i = 0;
    let mut str_buff: Vec<u8> = vec!();
    while i < str_len {
        if let Some(b) = bytes_iter.next() {
            str_buff.push(*b);
        } else {
            return Err(BencodeError::BytestreamEnded);
        }
        i = i + 1;
    }
    Ok(ByteString::new(str_buff))
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_bytes_eq {
        ($bytes:expr, $expected:expr) => {
            match parse_bytes(&mut $bytes.iter().peekable()) {
                Ok(r) => assert_eq!($expected, r),
                Err(e) => panic!("Unexpected err: {:?}", e)
            }
        };
    }

    macro_rules! assert_bytes_err {
        ($bytes:expr, $expected:expr) => {
            match parse_bytes(&mut $bytes.iter().peekable()) {
                Ok(e) => panic!("Unexpected ok: {:?}. Expected err: {:?}", e, $expected),
                Err(r) => assert_eq!($expected, r)
            }
        };
    }

    macro_rules! bencode_string {
        ($literal:expr) => {
            ByteString::new($literal.as_bytes().to_vec())
        };
    }

    #[test]
    fn dict() {
        assert_bytes_eq!(vec!(0x64, 0x65), BencodeItem::Dict(vec!()));

        assert_bytes_eq!(
            vec!(0x64, 0x35, 0x3A, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x35, 0x3A, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x35, 0x3A, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x35, 0x3A, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x65),
            BencodeItem::Dict(
                vec!(
                    (String::from("Hello"), BencodeItem::String(bencode_string!("World"))),
                    (String::from("World"), BencodeItem::String(bencode_string!("Hello")))
                )
            )
        );

        assert_bytes_eq!(
            vec!(0x64, 0x35, 0x3A, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x69, 0x31, 0x32, 0x33, 0x65, 0x65),
            BencodeItem::Dict(
                vec!((String::from("Hello"), BencodeItem::Int(123)))
            )
        );
    }

    #[test]
    fn list() {
        assert_bytes_eq!(vec!(0x6C, 0x65), BencodeItem::List(vec!()));
        assert_bytes_eq!(
            vec!(0x6C, 0x6C, 0x65, 0x65),
            BencodeItem::List(vec!(
                BencodeItem::List(vec!())
            ))
        );
        assert_bytes_eq!(
            vec!(0x6C, 0x35, 0x3A, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x65),
            BencodeItem::List(vec!(
                BencodeItem::String(bencode_string!("Hello"))
            ))
        );
        assert_bytes_eq!(
            vec!(0x6C, 0x30, 0x3A, 0x69, 0x31, 0x33, 0x33, 0x37, 0x65, 0x35, 0x3A, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x65),
            BencodeItem::List(vec!(
                BencodeItem::String(bencode_string!("")),
                BencodeItem::Int(1337),
                BencodeItem::String(bencode_string!("Hello"),
                )
            ))
        );
    }

    #[test]
    fn string() {
        assert_bytes_eq!(vec!(0x35, 0x3A, 0x48, 0x65, 0x6C, 0x6C, 0x6F), BencodeItem::String(bencode_string!("Hello")));
        assert_bytes_eq!(vec!(0x31, 0x31, 0x3A, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64), BencodeItem::String(bencode_string!("Hello World")));
        assert_bytes_eq!(vec!(0x30, 0x3A), BencodeItem::String(bencode_string!("")));
        assert_bytes_eq!(vec!(0x31, 0x3A, 0x78), BencodeItem::String(bencode_string!("x")));
        assert_bytes_err!(vec!(0x31, 0x30, 0x78, 0x3A, 0x7A), BencodeError::StrLenInvalidByte);
        assert_bytes_eq!(vec!(0x31, 0x3A, 0x8A), BencodeItem::String(ByteString::new(vec!(0x8A))));
        assert_bytes_err!(vec!(0x31, 0x30, 0x3A, 0x7A), BencodeError::BytestreamEnded);
    }

    #[test]
    fn int() {
        assert_bytes_eq!(vec!(0x69, 0x31, 0x33, 0x33, 0x37, 0x65), BencodeItem::Int(1337));
        assert_bytes_eq!(vec!(0x69, 0x37, 0x65), BencodeItem::Int(7));
        assert_bytes_eq!(vec!(0x69, 0x31, 0x36, 0x36, 0x33, 0x30, 0x32, 0x34, 0x32, 0x39, 0x33, 0x65), BencodeItem::Int(1663024293));
        assert_bytes_eq!(vec!(0x69, 0x2D, 0x37, 0x65), BencodeItem::Int(-7));
        assert_bytes_eq!(vec!(0x69, 0x30, 0x65), BencodeItem::Int(0));
        assert_bytes_err!(vec!(0x69, 0x2D, 0x30, 0x65), BencodeError::IntParseNegativeZero);
        assert_bytes_err!(vec!(0x69, 0x30, 0x30, 0x30, 0x65), BencodeError::IntParseLeadingZero);
        assert_bytes_err!(vec!(0x69, 0x30, 0x30, 0x31, 0x65), BencodeError::IntParseLeadingZero);
        assert_bytes_err!(vec!(0x69, 0x3A, 0x65), BencodeError::IntParseInt(format!("invalid digit found in string")));
        assert_bytes_err!(vec!(0x69, 0x65), BencodeError::UnexpectedEndMarker);
        assert_bytes_err!(vec!(0x65, 0x69), BencodeError::UnexpectedEndMarker);
    }
}
