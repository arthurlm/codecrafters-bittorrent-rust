use std::{
    collections::BTreeMap,
    ops::{AddAssign, MulAssign},
};

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("Bencode parse error: {0}")]
pub struct ParseError(String);

impl ParseError {
    pub fn new(msg: &str) -> Self {
        Self(msg.to_string())
    }
}

impl From<(&str, &[u8])> for ParseError {
    fn from((msg, input): (&str, &[u8])) -> Self {
        Self(format!("{msg}: {input:?}"))
    }
}

pub type IResult<'a, T> = Result<(&'a [u8], T), ParseError>;

fn parse_num<O>(input: &[u8], end_char: u8) -> IResult<O>
where
    O: Default + MulAssign<O> + AddAssign<O> + From<u8>,
{
    let mut idx = 0;
    let mut num = O::default();

    loop {
        if idx >= input.len() {
            return Err(("String num does not contains end tag", input).into());
        }

        match input[idx] {
            val if val == end_char && idx == 0 => {
                return Err(("Number cannot be empty", input).into())
            }
            val if val == end_char => return Ok((&input[idx + 1..], num)),
            val @ b'0'..=b'9' => {
                num *= O::from(10);
                num += O::from(val - b'0');
            }
            val => {
                return Err(ParseError::new(&format!(
                    "String num is not a number={val}: {input:?}"
                )))
            }
        }
        idx += 1;
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BencodeText(Vec<u8>);

impl BencodeText {
    pub fn parse(input: &[u8]) -> IResult<Self> {
        let (input, str_len) = parse_num(input, b':')?;
        if str_len > input.len() {
            return Err(("String payload is too short", input).into());
        }
        let (text, input) = input.split_at(str_len);
        Ok((input, Self(text.to_vec())))
    }
}

impl From<BencodeText> for String {
    fn from(value: BencodeText) -> Self {
        String::from_utf8_lossy(&value.0).to_string()
    }
}

impl From<BencodeText> for serde_json::Value {
    fn from(value: BencodeText) -> Self {
        Self::String(value.into())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BencodeValue {
    Data(BencodeText),
    Integer(i64),
    List(Vec<BencodeValue>),
    Dict(BTreeMap<BencodeText, BencodeValue>),
}

impl BencodeValue {
    pub fn parse(input: &[u8]) -> IResult<Self> {
        if input.is_empty() {
            return Err(("Input is empty", input).into());
        }

        match input[0] {
            b'0'..=b'9' => {
                let (input, text) = BencodeText::parse(input)?;
                Ok((input, Self::Data(text)))
            }
            b'i' => {
                if input.len() >= 2 && input[1] == b'-' {
                    let (input, num) = parse_num::<i64>(&input[2..], b'e')?;
                    Ok((input, Self::Integer(-num)))
                } else {
                    let (input, num) = parse_num(&input[1..], b'e')?;
                    Ok((input, Self::Integer(num)))
                }
            }
            b'l' => {
                let (input, items) = parse_list(&input[1..])?;
                Ok((input, Self::List(items)))
            }
            b'd' => {
                let (input, dict) = parse_dict(&input[1..])?;
                Ok((input, Self::Dict(dict)))
            }
            _ => Err(("Invalid Bencode content", input).into()),
        }
    }
}

impl From<BencodeValue> for serde_json::Value {
    fn from(value: BencodeValue) -> Self {
        match value {
            BencodeValue::Data(txt) => txt.into(),
            BencodeValue::Integer(num) => Self::Number(num.into()),
            BencodeValue::List(values) => {
                Self::Array(values.into_iter().map(|x| x.into()).collect())
            }
            BencodeValue::Dict(dict) => Self::Object(
                dict.into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            ),
        }
    }
}

fn parse_list(mut input: &[u8]) -> IResult<Vec<BencodeValue>> {
    let mut output = Vec::new();

    loop {
        if input.is_empty() {
            return Err(("List miss end tag", input).into());
        }

        if input[0] == b'e' {
            return Ok((&input[1..], output));
        }
        let (next_input, item) = BencodeValue::parse(input)?;
        input = next_input;
        output.push(item);
    }
}

fn parse_dict(mut input: &[u8]) -> IResult<BTreeMap<BencodeText, BencodeValue>> {
    let mut output = BTreeMap::new();

    loop {
        if input.is_empty() {
            return Err(("Dict miss end tag", input).into());
        }

        if input[0] == b'e' {
            return Ok((&input[1..], output));
        }
        let (next_input, key) = BencodeText::parse(input)?;
        let (next_input, item) = BencodeValue::parse(next_input)?;
        input = next_input;
        output.insert(key, item);
    }
}
