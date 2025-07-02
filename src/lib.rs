#![allow(dead_code)]

use colored::{Color, Colorize};
use std::iter;


const HEADER_LENGTH: usize = 12;

/// Compose basic Query
///
/// ID in the header is set to the provided value. The QR flag to 0 for query. OPCODE also to 0 for standard query.
/// RD is automatically se to 1, same as QDCOUNT.
///
/// The domain name is encoded to a sequence of labels. QTYPE and QCLASS are both set to 1.
///
/// # Examples
///
/// ```
/// let payload = dnsr::compose(45, "www.rust-lang.org");
/// ```
pub fn compose(id: u16, domain_name: &str) -> Vec<u8> {
    let length = domain_name.len() + 18;
    let mut payload = vec![0; length];
    set_id(&mut payload, id);
    set_rd(&mut payload, true);
    set_qdcount(&mut payload, 1);
    parse_question(&mut payload, &domain_name);
    payload
}

fn set_u16(payload: &mut [u8], index: usize, value: u16) {
    let value_bytes = value.to_be_bytes();
    payload[index] = value_bytes[0];
    payload[index + 1] = value_bytes[1];
}

fn get_u16(payload: &[u8], index: usize) -> u16 {
    let mut value_bytes = [0; 2];
    value_bytes.clone_from_slice(&payload[index..index+2]);
    u16::from_be_bytes(value_bytes)
}

fn set_id(payload: &mut Vec<u8>, id: u16) {
    set_u16(payload, 0, id);
}

/// Extract the ID field as a u16.
pub fn get_id(payload: &[u8]) -> u16 {
    get_u16(payload, 0)
}

pub fn get_qr(payload: &[u8]) -> u16 {
    const MASK: u8 = 0b10000000;
    (payload[2] & MASK >> 7).into()
}

fn set_rd(payload: &mut Vec<u8>, state: bool) {
    const MASK: u8 = 0b00000001;
    if state {
        payload[2] = payload[2] | MASK;
    } else {
        payload[2] &= !MASK;
    }
}

pub fn get_rd(payload: &[u8]) -> bool {
    const MASK: u8 = 0b10000000;
    payload[3] & MASK != 0
}

pub fn get_ra(payload: &[u8]) -> bool {
    const MASK: u8 = 0b00000001;
    payload[2] & MASK != 0
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseCode {
    Ok = 0,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
    NA
}

impl From<ResponseCode> for u8 {
    fn from(val: ResponseCode) -> Self {
        val as u8
    }
}

impl From<u8> for ResponseCode {
    fn from(val: u8) -> Self {
        match val {
            0 => ResponseCode::Ok,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            _ => ResponseCode::NA,
        }
    }
}

pub fn get_rcode(payload: &[u8]) -> ResponseCode {
    const MASK: u8 = 0b00001111;
    let rc_value: u8 = payload[3] & MASK;
    let res: ResponseCode = rc_value.into();
    res
}

fn set_qdcount(payload: &mut [u8], count: u16) {
    set_u16(payload, 4, count);
}

pub fn get_qdcount(payload: &[u8]) -> u16 {
    get_u16(payload, 4)
}

pub fn get_ancount(payload: &[u8]) -> u16 {
    get_u16(payload, 6)
}

fn parse_question(payload: &mut Vec<u8>, question: &str) {
    const QUESTION_START: usize = 12;
    let mut offset = 0;
    for part in question.split('.') {
        let size: u8 = part.len().try_into().unwrap();
        payload[QUESTION_START + offset] = size;
        offset += 1;
        for byte in part.as_bytes() {
            payload[QUESTION_START + offset] = *byte;
            offset += 1;
        }
    }
    payload[QUESTION_START + offset + 2] = 1;
    payload[QUESTION_START + offset + 4] = 1;
}

pub fn question_length(payload: &[u8]) -> usize {
    const QUESTION_START: usize = 12;
    let mut length = 0;
    let mut it = payload[QUESTION_START];
    while it != 0 {
        length += it + 1;
        it = payload[QUESTION_START + length as usize];
    }
    length += 5;
    length as usize
}

fn is_pointer(octet: u8) -> bool {
    const MASK: u8 = 0b11000000;
    octet & MASK == MASK
}

pub fn get_ips(payload: &[u8], mut index: usize) -> Vec<String> {
    let ancount = get_ancount(payload);
    let mut answers: Vec<String> = vec![];
    for _i in 0..ancount {
        let (answer, new_index) = get_ip(payload, index);
        answers.push(answer);
        index = new_index;
    }
    answers
}

fn get_ip(payload: &[u8], index: usize) -> (String, usize) {
    const RDLENGTH_OFFSET: usize = 10;
    let r_data_length = get_u16(payload, index + RDLENGTH_OFFSET);
    let r_data_index = index + RDLENGTH_OFFSET + 2;
    let answer = payload[r_data_index..(r_data_index + r_data_length as usize)]
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(".");
    let new_index = r_data_index + r_data_length as usize;
    (answer, new_index)
}

/// Print color-coded payload in hex.
///
/// Header section in blue, question in green and RR in answer in magenta.
pub fn print(payload: &[u8]) {

    fn print_(bytes: &[u8], color: colored::Color) {
        for byte in bytes {
            print!("{} ", format!("{byte:02X}").color(color));
        }
    }

    print_(&payload[0..HEADER_LENGTH], Color::Blue);   // Header
    let answer_index = question_length(payload) + HEADER_LENGTH;
    print_(&payload[12..answer_index], Color::Green);  // Question
    print_(&payload[answer_index..], Color::Magenta);  // Answer
    println!();

}

/// Print color-coded header with a legend in hex.
pub fn print_header(payload: &[u8]) {

    fn print_(bytes: &[u8], color: colored::Color) {
        for byte in &bytes[0..2] {
            print!("{} ", format!("{byte:02X}").color(color));
        }
    }

    // Print legend
    println!("=HEADER============================");
    println!("{}    {} {}   {}   {}   {}",
                "ID".blue(), "FLAGS".red(), "QDC".green(),
                "ANC".yellow(), "NSC".magenta(), "ARC".cyan());

    // Print payload
    print_(&payload[0..2], Color::Blue);      // ID
    print_(&payload[2..4], Color::Red);       // Flags
    print_(&payload[4..6], Color::Green);     // QDCOUNT
    print_(&payload[6..8], Color::Yellow);    // ANCOUNT
    print_(&payload[8..10], Color::Magenta);  // NSCOUNT
    print_(&payload[10..12], Color::Cyan);    // ARCOUNT
    println!();
    println!("===================================");

}

/// Print color-coded flags with a legend in binary.
pub fn print_flags(payload: &[u8]) {

    fn print_(string: &str, counts: &[usize], colors: &[Color]) {
        let mut it = string.chars();
        for (count, color) in iter::zip(counts, colors) {
            let data: String = it.by_ref().take(*count).collect();
            print!("{}", data.color(*color));
        }
        println!();
    }

    // Print legend
    println!("=FLAGS==========");
    let legend_1 = String::from("QO   ATRRZ  R   ");
    let legend_2 = String::from("RP   ACDA   C   ");
    // Print data
    let mut binary_string = String::new();
    for &byte in &payload[2..4] {
        binary_string.push_str(&format!("{:08b}", byte));
    }
    let counts = [1, 4, 1, 1, 1, 1, 3, 4];
    let colors = [Color::Blue, Color::Red, Color::White, Color::Green,
                                Color::Yellow, Color::Magenta, Color::White, Color::Cyan];
    print_(&legend_1, &counts, &colors);
    print_(&legend_2, &counts, &colors);
    print_(&binary_string, &counts, &colors);
    println!("================");

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn one_byte_id() {
        let id = 45;
        let payload = compose(id,&"");
        let gotten_id = get_id(&payload);
        assert_eq!(id, gotten_id);
    }

    #[test]
    fn two_byte_id() {
        let id = 999;
        let payload = compose(id,&"");
        let gotten_id = get_id(&payload);
        assert_eq!(id, gotten_id);
    }

    #[test]
    fn qdcount() {
        let payload = compose(45,&"");
        let count = get_qdcount(&payload);
        assert_eq!(count, 1);
    }

    #[test]
    fn parse_domain_name() {
        let domain_name = String::from("dns.google.com");
        let expected: [u8; 16] = [3, 100, 110, 115, 6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0];
        let payload = compose(45, &domain_name);
        assert_eq!(&payload[12..28], expected);
    }

    #[test]
    fn test_question_length() {
        let names = ["a.b.c", "dns.google.com", "www.seznam.cz",
            "www.some.page.com"];
        for name in names {
            let expected = name.len() + 6;
            let payload = compose(45, &name);
            assert_eq!(question_length(&payload), expected);
        }
    }

    // Test Response Code Enum
    #[test]
    fn get_u8_from_response_code() {
        let rc = ResponseCode::FormatError;
        let rc_value: u8 = rc.into();
        assert_eq!(rc_value, 1);
    }

    #[test]
    fn get_response_code_from_u8() {
        let rc_val: u8 = 2;
        let rc: ResponseCode = rc_val.into();
        assert_eq!(rc, ResponseCode::ServerFailure);
    }

}
