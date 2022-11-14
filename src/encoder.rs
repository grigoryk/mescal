use crate::{BencodeItem, c, ByteString};

pub trait AsBencodeBytes {
    fn as_bytes(self) -> Vec<u8>;
}

impl AsBencodeBytes for BencodeItem {
    fn as_bytes(self) -> Vec<u8> {
        match self {
            BencodeItem::String(s) => encode_string(s),
            BencodeItem::Int(i) => encode_int(i),
            BencodeItem::List(l) => encode_list(l),
            BencodeItem::Dict(d) => encode_dict(d),
        }
    }
}

fn encode_dict(d: Vec<(String, BencodeItem)>) -> Vec<u8> {
    let mut bytes = vec!(c::M_DICT);
    for (key, value) in d {
        bytes.append(&mut encode_string(ByteString::new(key.as_bytes().to_vec())));
        bytes.append(&mut value.as_bytes());
    }
    bytes.push(c::M_END);
    bytes
}

fn encode_list(l: Vec<BencodeItem>) -> Vec<u8> {
    let mut bytes = vec!(c::M_LIST);
    for item in l {
        bytes.append(&mut item.as_bytes());
    }
    bytes.push(c::M_END);
    bytes
}

fn encode_int(i: i64) -> Vec<u8> {
    let mut bytes = vec!(c::M_INT);
    bytes.append(&mut i.to_string().into_bytes());
    bytes.push(c::M_END);
    bytes
}

fn encode_string(mut s: ByteString) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec!();
    bytes.append(&mut s.bytes.len().to_string().into_bytes());
    bytes.push(c::M_COLON);
    bytes.append(&mut s.bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_bytes_eq {
        ($encoded:expr, $decoded:expr) => {
            assert_eq!($encoded, $decoded.as_bytes());
        };
    }

    macro_rules! bencode_string {
        ($literal:expr) => {
            ByteString::new($literal.as_bytes().to_vec())
        };
    }

    #[test]
    fn read_dict() {
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
    fn read_list() {
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
    fn read_string() {
        assert_bytes_eq!(vec!(0x35, 0x3A, 0x48, 0x65, 0x6C, 0x6C, 0x6F), BencodeItem::String(bencode_string!("Hello")));
        assert_bytes_eq!(vec!(0x31, 0x31, 0x3A, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64), BencodeItem::String(bencode_string!("Hello World")));
        assert_bytes_eq!(vec!(0x30, 0x3A), BencodeItem::String(bencode_string!("")));
        assert_bytes_eq!(vec!(0x31, 0x3A, 0x78), BencodeItem::String(bencode_string!("x")));
        assert_bytes_eq!(vec!(0x31, 0x3A, 0x8A), BencodeItem::String(ByteString::new(vec!(0x8A))));
    }

    #[test]
    fn read_int() {
        assert_bytes_eq!(vec!(0x69, 0x31, 0x33, 0x33, 0x37, 0x65), BencodeItem::Int(1337));
        assert_bytes_eq!(vec!(0x69, 0x37, 0x65), BencodeItem::Int(7));
        assert_bytes_eq!(vec!(0x69, 0x31, 0x36, 0x36, 0x33, 0x30, 0x32, 0x34, 0x32, 0x39, 0x33, 0x65), BencodeItem::Int(1663024293));
        assert_bytes_eq!(vec!(0x69, 0x2D, 0x37, 0x65), BencodeItem::Int(-7));
        assert_bytes_eq!(vec!(0x69, 0x30, 0x65), BencodeItem::Int(0));
    }
}