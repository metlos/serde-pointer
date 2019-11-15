/*
 *   Copyright (c) 2019 Lukas Krejci
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

use crate::pointer::{Pointer, Step};
use nom::{
    self,
    branch::alt,
    character::complete::{anychar, char, digit1, none_of},
    combinator::{all_consuming, opt},
    error::{convert_error, ParseError as NomParseError, VerboseError},
    multi::many0,
    sequence::preceded,
    IResult,
};
use std::fmt;
use std::fmt::Display;
use std::str;

pub(crate) fn parse(s: &str) -> Result<Pointer, ParseError> {
    let result = _parse(s);

    match result {
        Ok(r) => Ok(r.1),
        Err(e) => Err(ParseError {
            error: match e {
                nom::Err::Incomplete(_) => "Incomplete JSON Pointer".to_owned(),
                nom::Err::Error(e) => convert_error(s, VerboseError::from_error_kind(e.0, e.1)),
                nom::Err::Failure(e) => convert_error(s, VerboseError::from_error_kind(e.0, e.1)),
            },
        }),
    }
}

fn _parse(s: &str) -> IResult<&str, Pointer> {
    let (s, _) = opt(char('#'))(s)?;
    let (s, segments) = all_consuming(many0(preceded(char('/'), _parse_segment)))(s)?;

    Ok((s, segments.into()))
}

fn _parse_segment(s: &str) -> IResult<&str, Step> {
    if s.is_empty() {
        return Ok((s, Step::Name(String::new())));
    }

    alt((_parse_index, _parse_new_element, _parse_name))(s)
}

fn _parse_index(s: &str) -> IResult<&str, Step> {
    let leading_zero = char::<_, (&str, nom::error::ErrorKind)>('0')(s);
    match leading_zero {
        Ok((rest, _)) => {
            if segment_ends(rest) {
                Ok((rest, Step::Index(0)))
            } else {
                _parse_name(s)
            }
        },
        Err(_) => {
            // not a leading 0
            let (s, ds) = digit1(s)?;
            let idx = ds
                .parse::<usize>()
                .map_err(|_| nom::Err::Error(nom::error::make_error(s, nom::error::ErrorKind::Digit)))?;
            Ok((s, Step::Index(idx)))
        }
    }
}

fn _parse_name(s: &str) -> IResult<&str, Step> {
    let (s, cs) = many0(_escape_seq_or_char)(s)?;
    Ok((s, Step::Name(cs.into_iter().collect())))
}

fn _parse_new_element(s: &str) -> IResult<&str, Step> {
    match char('-')(s) {
        Ok((rest, _)) => {
            if segment_ends(rest) {
                Ok((rest, Step::NewElement))
            } else {
                _parse_name(s)
            }
        },
        Err(e) => Err(e)
    }
}

fn _escape_seq_or_char(s: &str) -> IResult<&str, char> {
    let escape_check = _escape_sequence(s);
    match escape_check {
        Ok(r) => Ok(r),
        Err(e) => {
            match e {
                // propagate the invalid escape sequence error
                nom::Err::Error((_, nom::error::ErrorKind::Escaped)) => Err(e),
                // otherwise this is not an escape sequence
                _ => none_of("/")(s)
            }
        }
    }
}

fn _escape_sequence(s: &str) -> IResult<&str, char> {
    let (s, _) = char('~')(s)?;
    let (s, l) = anychar(s)?;
    match l {
        '0' => Ok((s, '~')),
        '1' => Ok((s, '/')),
        _ => Err(nom::Err::Error(nom::error::make_error(s, nom::error::ErrorKind::Escaped)))
    }
}

fn segment_ends(s: &str) -> bool {
    s.is_empty() || s.chars().nth(0).unwrap() == '/'
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub error: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Invalid JSON Pointer: {}", self.error))
    }
}

impl nom::error::ParseError<&str> for ParseError {
    fn from_error_kind(_: &str, kind: nom::error::ErrorKind) -> Self {
        Self {
            error: kind.description().to_owned(),
        }
    }

    fn append(_: &str, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_hash() {
        let p = test_parse("#");
        assert_eq!(0, p.len());
    }

    #[test]
    fn empty_trailing_name() {
        let p = test_parse("#/");
        assert_eq!(1, p.len());
        assert_eq!(Step::Name("".to_owned()), p[0]);
    }

    #[test]
    fn empty_name_in_middle() {
        let p = test_parse("#//");
        assert_eq!(2, p.len());
        assert_eq!(Step::Name("".to_owned()), p[0]);
        assert_eq!(Step::Name("".to_owned()), p[1]);
    }

    #[test]
    fn index() {
        let p = test_parse("/21");
        assert_eq!(1, p.len());
        assert_eq!(Step::Index(21), p[0]);
    }

    #[test]
    fn leading_zeros_as_string() {
        let p = test_parse("/007");
        assert_eq!(Step::Name("007".to_owned()), p[0]);
    }

    #[test]
    fn escape_tilda() {
        let p = test_parse("/a~0/~0b/c~0d");
        assert_eq!(3, p.len());
        assert_eq!(Step::Name("a~".to_owned()), p[0]);
        assert_eq!(Step::Name("~b".to_owned()), p[1]);
        assert_eq!(Step::Name("c~d".to_owned()), p[2]);
    }

    #[test]
    fn escape_slash() {
        let p = test_parse("/a~1/~1b/c~1d");
        assert_eq!(3, p.len());
        assert_eq!(Step::Name("a/".to_owned()), p[0]);
        assert_eq!(Step::Name("/b".to_owned()), p[1]);
        assert_eq!(Step::Name("c/d".to_owned()), p[2]);
    }

    #[test]
    fn fails_on_unknown_escape_seq() {
        let r = parse("/a~2");
        assert!(r.is_err())
    }

    fn test_parse(s: &str) -> Vec<Step> {
        parse(s).unwrap().into()
    }
}
