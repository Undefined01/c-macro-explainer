use logos::Logos;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::multispace1,
    combinator::map,
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};
use std::collections::HashMap;

#[derive(Logos, Debug, PartialEq)]
enum Token {
    #[regex(r"[\p{XID_Start}_]\p{XID_Continue}*")]
    Identifier,

    #[token("\r\n")]
    #[token("\r")]
    #[token("\n")]
    LineTerminator,

    #[token("#")]
    Hash,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token(",")]
    Comma,

    #[regex(r"[ \t]+", logos::skip)]
    Whitespace,
}
