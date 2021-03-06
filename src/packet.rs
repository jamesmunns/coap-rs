use bincode;
use std::collections::BTreeMap;
use std::collections::LinkedList;

macro_rules! u8_to_unsigned_be {
	($src:ident, $start:expr, $end:expr, $t:ty) => ({
		(0 .. $end - $start + 1).rev().fold(0, |acc, i| acc | $src[$start+i] as $t << i * 8)
	})
}

#[derive(PartialEq, Eq, Debug)]
pub enum PacketType {
    Confirmable,
    NonConfirmable,
    Acknowledgement,
    Reset,
    Invalid,
}

#[derive(Default, Debug, RustcEncodable, RustcDecodable)]
pub struct PacketHeaderRaw {
    ver_type_tkl: u8,
    code: u8,
    message_id: u16,
}

#[derive(Debug)]
pub struct PacketHeader {
    ver_type_tkl: u8,
    pub code: PacketClass,
    message_id: u16,
}

#[derive(Debug, PartialEq)]
pub enum PacketClass {
    Empty,
    Request(Requests),
    Response(Responses),
    Reserved,
}

#[derive(Debug, PartialEq)]
pub enum Requests {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Debug, PartialEq)]
pub enum Responses {
    // 200 Codes
    Created,
    Deleted,
    Valid,
    Changed,
    Content,

    // 400 Codes
    BadRequest,
    Unauthorized,
    BadOption,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    PreconditionFailed,
    RequestEntityTooLarge,
    UnsupportedContentFormat,

    // 500 Codes
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    ProxyingNotSupported,
}

pub fn class_to_code(class: &PacketClass) -> u8 {
    return match *class {
        PacketClass::Empty => 0x00,

        PacketClass::Request(Requests::Get) => 0x01,
        PacketClass::Request(Requests::Post) => 0x02,
        PacketClass::Request(Requests::Put) => 0x03,
        PacketClass::Request(Requests::Delete) => 0x04,

        PacketClass::Response(Responses::Created) => 0x41,
        PacketClass::Response(Responses::Deleted) => 0x42,
        PacketClass::Response(Responses::Valid) => 0x43,
        PacketClass::Response(Responses::Changed) => 0x44,
        PacketClass::Response(Responses::Content) => 0x45,

        PacketClass::Response(Responses::BadRequest) => 0x80,
        PacketClass::Response(Responses::Unauthorized) => 0x81,
        PacketClass::Response(Responses::BadOption) => 0x82,
        PacketClass::Response(Responses::Forbidden) => 0x83,
        PacketClass::Response(Responses::NotFound) => 0x84,
        PacketClass::Response(Responses::MethodNotAllowed) => 0x85,
        PacketClass::Response(Responses::NotAcceptable) => 0x86,
        PacketClass::Response(Responses::PreconditionFailed) => 0x8C,
        PacketClass::Response(Responses::RequestEntityTooLarge) => 0x8D,
        PacketClass::Response(Responses::UnsupportedContentFormat) => 0x8F,

        PacketClass::Response(Responses::InternalServerError) => 0x90,
        PacketClass::Response(Responses::NotImplemented) => 0x91,
        PacketClass::Response(Responses::BadGateway) => 0x92,
        PacketClass::Response(Responses::ServiceUnavailable) => 0x93,
        PacketClass::Response(Responses::GatewayTimeout) => 0x94,
        PacketClass::Response(Responses::ProxyingNotSupported) => 0x95,

        _ => 0xFF,
    } as u8;
}

pub fn code_to_class(code: &u8) -> PacketClass {
    match *code {
        0x00 => PacketClass::Empty,

        0x01 => PacketClass::Request(Requests::Get),
        0x02 => PacketClass::Request(Requests::Post),
        0x03 => PacketClass::Request(Requests::Put),
        0x04 => PacketClass::Request(Requests::Delete),

        0x41 => PacketClass::Response(Responses::Created),
        0x42 => PacketClass::Response(Responses::Deleted),
        0x43 => PacketClass::Response(Responses::Valid),
        0x44 => PacketClass::Response(Responses::Changed),
        0x45 => PacketClass::Response(Responses::Content),

        0x80 => PacketClass::Response(Responses::BadRequest),
        0x81 => PacketClass::Response(Responses::Unauthorized),
        0x82 => PacketClass::Response(Responses::BadOption),
        0x83 => PacketClass::Response(Responses::Forbidden),
        0x84 => PacketClass::Response(Responses::NotFound),
        0x85 => PacketClass::Response(Responses::MethodNotAllowed),
        0x86 => PacketClass::Response(Responses::NotAcceptable),
        0x8C => PacketClass::Response(Responses::PreconditionFailed),
        0x8D => PacketClass::Response(Responses::RequestEntityTooLarge),
        0x8F => PacketClass::Response(Responses::UnsupportedContentFormat),

        0x90 => PacketClass::Response(Responses::InternalServerError),
        0x91 => PacketClass::Response(Responses::NotImplemented),
        0x92 => PacketClass::Response(Responses::BadGateway),
        0x93 => PacketClass::Response(Responses::ServiceUnavailable),
        0x94 => PacketClass::Response(Responses::GatewayTimeout),
        0x95 => PacketClass::Response(Responses::ProxyingNotSupported),

        _ => PacketClass::Reserved,
    }
}

pub fn code_to_str(code: &u8) -> String {
    let class_code = (0xE0 & code) >> 5;
    let detail_code = 0x1F & code;

    return format!("{}.{:02}", class_code, detail_code);
}

pub fn class_to_str(class: &PacketClass) -> String {
    return code_to_str(&class_to_code(class));
}

impl PacketHeader {
    pub fn new() -> PacketHeader {
        return PacketHeader::from_raw(&PacketHeaderRaw::default());
    }

    pub fn from_raw(raw: &PacketHeaderRaw) -> PacketHeader {
        return PacketHeader {
            ver_type_tkl: raw.ver_type_tkl,
            code: code_to_class(&raw.code),
            message_id: raw.message_id,
        };
    }

    pub fn to_raw(&self) -> PacketHeaderRaw {
        return PacketHeaderRaw {
            ver_type_tkl: self.ver_type_tkl,
            code: class_to_code(&self.code),
            message_id: self.message_id,
        };
    }

    #[inline]
    pub fn set_version(&mut self, v: u8) {
        let type_tkl = 0x3F & self.ver_type_tkl;
        self.ver_type_tkl = v << 6 | type_tkl;
    }

    #[inline]
    pub fn get_version(&self) -> u8 {
        return self.ver_type_tkl >> 6;
    }

    #[inline]
    pub fn set_type(&mut self, t: PacketType) {
        let tn = match t {
            PacketType::Confirmable => 0,
            PacketType::NonConfirmable => 1,
            PacketType::Acknowledgement => 2,
            PacketType::Reset => 3,
            _ => unreachable!(),
        };

        let ver_tkl = 0xCF & self.ver_type_tkl;
        self.ver_type_tkl = tn << 4 | ver_tkl;
    }

    #[inline]
    pub fn get_type(&self) -> PacketType {
        let tn = (0x30 & self.ver_type_tkl) >> 4;
        match tn {
            0 => PacketType::Confirmable,
            1 => PacketType::NonConfirmable,
            2 => PacketType::Acknowledgement,
            3 => PacketType::Reset,
            _ => PacketType::Invalid,
        }
    }

    #[inline]
    fn set_token_length(&mut self, tkl: u8) {
        assert_eq!(0xF0 & tkl, 0);

        let ver_type = 0xF0 & self.ver_type_tkl;
        self.ver_type_tkl = tkl | ver_type;
    }

    #[inline]
    fn get_token_length(&self) -> u8 {
        return 0x0F & self.ver_type_tkl;
    }

    pub fn set_code(&mut self, code: &str) {
        let code_vec: Vec<&str> = code.split('.').collect();
        assert_eq!(code_vec.len(), 2);

        let class_code = code_vec[0].parse::<u8>().unwrap();
        let detail_code = code_vec[1].parse::<u8>().unwrap();
        assert_eq!(0xF8 & class_code, 0);
        assert_eq!(0xE0 & detail_code, 0);

        self.code = code_to_class(&(class_code << 5 | detail_code));
    }

    pub fn get_code(&self) -> String {
        class_to_str(&self.code)
    }

    #[inline]
    pub fn set_message_id(&mut self, message_id: u16) {
        self.message_id = message_id;
    }

    #[inline]
    pub fn get_message_id(&self) -> u16 {
        return self.message_id;
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidHeader,
    InvalidTokenLength,
    InvalidOptionDelta,
    InvalidOptionLength,
}

#[derive(Debug)]
pub enum PackageError {
    InvalidHeader,
    InvalidPacketLength,
}

#[derive(PartialEq, Eq, Debug)]
pub enum OptionType {
    IfMatch,
    UriHost,
    ETag,
    IfNoneMatch,
    Observe,
    UriPort,
    LocationPath,
    UriPath,
    ContentFormat,
    MaxAge,
    UriQuery,
    Accept,
    LocationQuery,
    Block2,
    Block1,
    ProxyUri,
    ProxyScheme,
    Size1,
}

#[derive(Debug)]
pub struct Packet {
    pub header: PacketHeader,
    token: Vec<u8>,
    options: BTreeMap<usize, LinkedList<Vec<u8>>>,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn new() -> Packet {
        Packet {
            header: PacketHeader::new(),
            token: Vec::new(),
            options: BTreeMap::new(),
            payload: Vec::new(),
        }
    }

    pub fn set_token(&mut self, token: Vec<u8>) {
        self.header.set_token_length(token.len() as u8);
        self.token = token;
    }

    pub fn get_token(&self) -> &Vec<u8> {
        return &self.token;
    }

    pub fn set_option(&mut self, tp: OptionType, value: LinkedList<Vec<u8>>) {
        let num = Self::get_option_number(tp);
        self.options.insert(num, value);
    }

    pub fn set_payload(&mut self, payload: Vec<u8>) {
        self.payload = payload;
    }

    pub fn add_option(&mut self, tp: OptionType, value: Vec<u8>) {
        let num = Self::get_option_number(tp);
        match self.options.get_mut(&num) {
            Some(list) => {
                list.push_back(value);
                return;
            }
            None => (),
        };

        let mut list = LinkedList::new();
        list.push_back(value);
        self.options.insert(num, list);
    }

    pub fn get_option(&self, tp: OptionType) -> Option<LinkedList<Vec<u8>>> {
        let num = Self::get_option_number(tp);
        match self.options.get(&num) {
            Some(options) => Some(options.clone()),
            None => None,
        }
    }

    /// Decodes a byte slice and construct the equivalent Packet.
    pub fn from_bytes(buf: &[u8]) -> Result<Packet, ParseError> {
        let header_result: bincode::DecodingResult<PacketHeaderRaw> = bincode::decode(buf);
        match header_result {
            Ok(raw_header) => {
                let header = PacketHeader::from_raw(&raw_header);
                let token_length = header.get_token_length();
                let options_start: usize = 4 + token_length as usize;

                if token_length > 8 {
                    return Err(ParseError::InvalidTokenLength);
                }

                if options_start > buf.len() {
                    return Err(ParseError::InvalidTokenLength);
                }

                let token = buf[4..options_start].to_vec();

                let mut idx = options_start;
                let mut options_number = 0;
                let mut options: BTreeMap<usize, LinkedList<Vec<u8>>> = BTreeMap::new();
                while idx < buf.len() {
                    let byte = buf[idx];

                    if byte == 255 || idx > buf.len() {
                        break;
                    }

                    let mut delta = (byte >> 4) as usize;
                    let mut length = (byte & 0xF) as usize;

                    idx += 1;

                    // Check for special delta characters
                    match delta {
                        13 => {
                            if idx >= buf.len() {
                                return Err(ParseError::InvalidOptionLength);
                            }
                            delta = buf[idx] as usize + 13;
                            idx += 1;
                        }
                        14 => {
                            if idx + 1 >= buf.len() {
                                return Err(ParseError::InvalidOptionLength);
                            }

                            delta = (u16::from_be(u8_to_unsigned_be!(buf, idx, idx + 1, u16)) +
                                     269) as usize;
                            idx += 2;
                        }
                        15 => {
                            return Err(ParseError::InvalidOptionDelta);
                        }
                        _ => {}
                    };

                    // Check for special length characters
                    match length {
                        13 => {
                            if idx >= buf.len() {
                                return Err(ParseError::InvalidOptionLength);
                            }

                            length = buf[idx] as usize + 13;
                            idx += 1;
                        }
                        14 => {
                            if idx + 1 >= buf.len() {
                                return Err(ParseError::InvalidOptionLength);
                            }

                            length = (u16::from_be(u8_to_unsigned_be!(buf, idx, idx + 1, u16)) +
                                      269) as usize;
                            idx += 2;
                        }
                        15 => {
                            return Err(ParseError::InvalidOptionLength);
                        }
                        _ => {}
                    };

                    options_number += delta;

                    let end = idx + length;
                    if end > buf.len() {
                        return Err(ParseError::InvalidOptionLength);
                    }
                    let options_value = buf[idx..end].to_vec();

                    if options.contains_key(&options_number) {
                        let mut options_list = options.get_mut(&options_number).unwrap();
                        options_list.push_back(options_value);
                    } else {
                        let mut list = LinkedList::new();
                        list.push_back(options_value);
                        options.insert(options_number, list);
                    }

                    idx += length;
                }

                let mut payload = Vec::new();
                if idx < buf.len() {
                    payload = buf[(idx + 1)..buf.len()].to_vec();
                }


                Ok(Packet {
                    header: header,
                    token: token,
                    options: options,
                    payload: payload,
                })
            }
            Err(_) => Err(ParseError::InvalidHeader),
        }
    }

    /// Returns a vector of bytes representing the Packet.
    pub fn to_bytes(&self) -> Result<Vec<u8>, PackageError> {
        let mut options_delta_length = 0;
        let mut options_bytes: Vec<u8> = Vec::new();
        for (number, value_list) in self.options.iter() {
            for value in value_list.iter() {
                let mut header: Vec<u8> = Vec::with_capacity(1 + 2 + 2);
                let delta = number - options_delta_length;

                let mut byte: u8 = 0;
                if delta <= 12 {
                    byte |= (delta << 4) as u8;
                } else if delta < 269 {
                    byte |= 13 << 4;
                } else {
                    byte |= 14 << 4;
                }
                if value.len() <= 12 {
                    byte |= value.len() as u8;
                } else if value.len() < 269 {
                    byte |= 13;
                } else {
                    byte |= 14;
                }
                header.push(byte);

                if delta > 12 && delta < 269 {
                    header.push((delta - 13) as u8);
                } else if delta >= 269 {
                    let fix = (delta - 269) as u16;
                    header.push((fix >> 8) as u8);
                    header.push((fix & 0xFF) as u8);
                }

                if value.len() > 12 && value.len() < 269 {
                    header.push((value.len() - 13) as u8);
                } else if value.len() >= 269 {
                    let fix = (value.len() - 269) as u16;
                    header.push((fix >> 8) as u8);
                    header.push((fix & 0xFF) as u8);
                }

                options_delta_length += delta;

                options_bytes.reserve(header.len() + value.len());
                unsafe {
                    use std::ptr;
                    let buf_len = options_bytes.len();
                    ptr::copy(header.as_ptr(),
                              options_bytes.as_mut_ptr().offset(buf_len as isize),
                              header.len());
                    ptr::copy(value.as_ptr(),
                              options_bytes.as_mut_ptr().offset((buf_len + header.len()) as isize),
                              value.len());
                    options_bytes.set_len(buf_len + header.len() + value.len());
                }
            }
        }

        let mut buf_length = 4 + self.payload.len() + self.token.len();
        if self.header.code != PacketClass::Empty && self.payload.len() != 0 {
            buf_length += 1;
        }
        buf_length += options_bytes.len();

        if buf_length > 1280 {
            return Err(PackageError::InvalidPacketLength);
        }

        let mut buf: Vec<u8> = Vec::with_capacity(buf_length);
        let header_result: bincode::EncodingResult<()> =
            bincode::encode_into(&self.header.to_raw(),
                                 &mut buf,
                                 bincode::SizeLimit::Infinite);
        match header_result {
            Ok(_) => {
                buf.reserve(self.token.len() + options_bytes.len());
                unsafe {
                    use std::ptr;
                    let buf_len = buf.len();
                    ptr::copy(self.token.as_ptr(),
                              buf.as_mut_ptr().offset(buf_len as isize),
                              self.token.len());
                    ptr::copy(options_bytes.as_ptr(),
                              buf.as_mut_ptr().offset((buf_len + self.token.len()) as isize),
                              options_bytes.len());
                    buf.set_len(buf_len + self.token.len() + options_bytes.len());
                }

                if self.header.code != PacketClass::Empty && self.payload.len() != 0 {
                    buf.push(0xFF);
                    buf.reserve(self.payload.len());
                    unsafe {
                        use std::ptr;
                        let buf_len = buf.len();
                        ptr::copy(self.payload.as_ptr(),
                                  buf.as_mut_ptr().offset(buf.len() as isize),
                                  self.payload.len());
                        buf.set_len(buf_len + self.payload.len());
                    }
                }
                Ok(buf)
            }
            Err(_) => Err(PackageError::InvalidHeader),
        }
    }

    fn get_option_number(tp: OptionType) -> usize {
        match tp {
            OptionType::IfMatch => 1,
            OptionType::UriHost => 3,
            OptionType::ETag => 4,
            OptionType::IfNoneMatch => 5,
            OptionType::Observe => 6,
            OptionType::UriPort => 7,
            OptionType::LocationPath => 8,
            OptionType::UriPath => 11,
            OptionType::ContentFormat => 12,
            OptionType::MaxAge => 14,
            OptionType::UriQuery => 15,
            OptionType::Accept => 17,
            OptionType::LocationQuery => 20,
            OptionType::Block2 => 23,
            OptionType::Block1 => 27,
            OptionType::ProxyUri => 35,
            OptionType::ProxyScheme => 39,
            OptionType::Size1 => 60,
        }
    }
}

/// Convert a request to a response
pub fn auto_response(request_packet: &Packet) -> Option<Packet> {
    let mut packet = Packet::new();

    packet.header.set_version(1);
    let response_type = match request_packet.header.get_type() {
        PacketType::Confirmable => PacketType::Acknowledgement,
        PacketType::NonConfirmable => PacketType::NonConfirmable,
        _ => return None,
    };
    packet.header.set_type(response_type);
    packet.header.code = PacketClass::Response(Responses::Content);
    packet.header.set_message_id(request_packet.header.get_message_id());
    packet.set_token(request_packet.get_token().clone());

    packet.payload = request_packet.payload.clone();

    Some(packet)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::LinkedList;

    #[test]
    fn test_header_codes() {
        for code in 0..255 {
            let class = code_to_class(&code);
            let code_str = code_to_str(&code);
            let class_str = class_to_str(&class);

            // Reserved class could technically be many codes
            //   so only check valid items
            if class != PacketClass::Reserved {
                assert_eq!(class_to_code(&class), code);
                assert_eq!(code_str, class_str);
            }
        }
    }

    #[test]
    fn test_decode_packet_with_options() {
        let buf = [0x44, 0x01, 0x84, 0x9e, 0x51, 0x55, 0x77, 0xe8, 0xb2, 0x48, 0x69, 0x04, 0x54,
                   0x65, 0x73, 0x74, 0x43, 0x61, 0x3d, 0x31];
        let packet = Packet::from_bytes(&buf);
        assert!(packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(packet.header.get_version(), 1);
        assert_eq!(packet.header.get_type(), PacketType::Confirmable);
        assert_eq!(packet.header.get_token_length(), 4);
        assert_eq!(packet.header.code, PacketClass::Request(Requests::Get));
        assert_eq!(packet.header.get_message_id(), 33950);
        assert_eq!(*packet.get_token(), vec![0x51, 0x55, 0x77, 0xE8]);
        assert_eq!(packet.options.len(), 2);

        let uri_path = packet.get_option(OptionType::UriPath);
        assert!(uri_path.is_some());
        let uri_path = uri_path.unwrap();
        let mut expected_uri_path = LinkedList::new();
        expected_uri_path.push_back("Hi".as_bytes().to_vec());
        expected_uri_path.push_back("Test".as_bytes().to_vec());
        assert_eq!(uri_path, expected_uri_path);

        let uri_query = packet.get_option(OptionType::UriQuery);
        assert!(uri_query.is_some());
        let uri_query = uri_query.unwrap();
        let mut expected_uri_query = LinkedList::new();
        expected_uri_query.push_back("a=1".as_bytes().to_vec());
        assert_eq!(uri_query, expected_uri_query);
    }

    #[test]
    fn test_decode_packet_with_payload() {
        let buf = [0x64, 0x45, 0x13, 0xFD, 0xD0, 0xE2, 0x4D, 0xAC, 0xFF, 0x48, 0x65, 0x6C, 0x6C,
                   0x6F];
        let packet = Packet::from_bytes(&buf);
        assert!(packet.is_ok());
        let packet = packet.unwrap();
        assert_eq!(packet.header.get_version(), 1);
        assert_eq!(packet.header.get_type(), PacketType::Acknowledgement);
        assert_eq!(packet.header.get_token_length(), 4);
        assert_eq!(packet.header.code,
                   PacketClass::Response(Responses::Content));
        assert_eq!(packet.header.get_message_id(), 5117);
        assert_eq!(*packet.get_token(), vec![0xD0, 0xE2, 0x4D, 0xAC]);
        assert_eq!(packet.payload, "Hello".as_bytes().to_vec());
    }

    #[test]
    fn test_encode_packet_with_options() {
        let mut packet = Packet::new();
        packet.header.set_version(1);
        packet.header.set_type(PacketType::Confirmable);
        packet.header.code = PacketClass::Request(Requests::Get);
        packet.header.set_message_id(33950);
        packet.set_token(vec![0x51, 0x55, 0x77, 0xE8]);
        packet.add_option(OptionType::UriPath, b"Hi".to_vec());
        packet.add_option(OptionType::UriPath, b"Test".to_vec());
        packet.add_option(OptionType::UriQuery, b"a=1".to_vec());
        assert_eq!(packet.to_bytes().unwrap(),
                   vec![0x44, 0x01, 0x84, 0x9e, 0x51, 0x55, 0x77, 0xe8, 0xb2, 0x48, 0x69, 0x04,
                        0x54, 0x65, 0x73, 0x74, 0x43, 0x61, 0x3d, 0x31]);
    }

    #[test]
    fn test_encode_packet_with_payload() {
        let mut packet = Packet::new();
        packet.header.set_version(1);
        packet.header.set_type(PacketType::Acknowledgement);
        packet.header.code = PacketClass::Response(Responses::Content);
        packet.header.set_message_id(5117);
        packet.set_token(vec![0xD0, 0xE2, 0x4D, 0xAC]);
        packet.payload = "Hello".as_bytes().to_vec();
        assert_eq!(packet.to_bytes().unwrap(),
                   vec![0x64, 0x45, 0x13, 0xFD, 0xD0, 0xE2, 0x4D, 0xAC, 0xFF, 0x48, 0x65, 0x6C,
                        0x6C, 0x6F]);
    }

    #[test]
    fn test_malicious_packet() {
        use rand;
        use quickcheck::{QuickCheck, StdGen, TestResult};

        fn run(x: Vec<u8>) -> TestResult {
            match Packet::from_bytes(&x[..]) {
                Ok(packet) => {
                    TestResult::from_bool(packet.get_token().len() ==
                                          packet.header.get_token_length() as usize)
                }
                Err(_) => TestResult::passed(),
            }
        }
        QuickCheck::new()
            .tests(10000)
            .gen(StdGen::new(rand::thread_rng(), 1500))
            .quickcheck(run as fn(Vec<u8>) -> TestResult)
    }
}
