use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alpha1, alphanumeric1, multispace0, multispace1},
    combinator::{opt, recognize, value},
    multi::{many0_count, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))
    .parse(input)
}

pub fn parse_define<'a>(input: &'a str) -> IResult<&'a str, (&'a str, Macro)> {
    let define_directive = tuple((tag("#"), multispace0, tag("define"), multispace1));
    let parameter_list = delimited(
        tag("("),
        separated_list0(
            tag(","),
            delimited(multispace0, alt((tag("..."), identifier)), multispace0),
        ),
        tag(")"),
    );
    let body = |input: &'a str| {
        let mut is_escaped = false;
        let mut is_new_line = false;
        let mut eatten = String::new();
        for (i, ch) in input.char_indices() {
            if ch == '\\' {
                is_escaped = !is_escaped;
            } else if is_escaped {
                is_escaped = false;
                if ch == '\n' {
                    is_new_line = true;
                } else {
                    eatten.push('\\');
                    eatten.push(ch);
                }
            } else if is_new_line && ch == ' ' {
                continue;
            } else if ch == '\n' && !is_escaped {
                return IResult::Ok((&input[i..], eatten.trim().to_string()));
            } else {
                is_new_line = false;
                eatten.push(ch);
            }
        }
        IResult::Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Eof,
        )))
    };
    preceded(
        define_directive,
        tuple((
            terminated(identifier, multispace0),
            opt(parameter_list),
            body,
        )),
    )
    .map(|(name, params, body): (&str, Option<Vec<&str>>, String)| {
        if let Some(params) = params {
            (
                name,
                Macro::Function {
                    params: params.into_iter().map(String::from).collect(),
                    body,
                },
            )
        } else {
            (name, Macro::Object { body })
        }
    })
    .parse(input)
}

pub fn parse_undef(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((tag("#"), multispace0, tag("undef"), multispace1)),
        identifier,
    )
    .parse(input)
}

pub fn parse_stringify(input: &str) -> IResult<&str, &str> {
    preceded(tuple((tag("#"), multispace0)), identifier).parse(input)
}

pub fn parse_concatenation(input: &str) -> IResult<&str, (&str, &str)> {
    tuple((
        identifier,
        delimited(multispace0, tag("##"), multispace0),
        identifier,
    ))
    .map(|(lhs, _, rhs)| (lhs, rhs))
    .parse(input)
}

pub fn parse_comment(input: &str) -> IResult<&str, ()> {
    alt((
        value((), tuple((tag("//"), take_until("\n")))),
        value((), tuple((tag("/*"), take_until("*/")))),
    ))
    .parse(input)
}

pub fn parse_macro_call<'a>(input: &'a str) -> IResult<&'a str, (&'a str, Vec<&'a str>)> {
    let argument = |input: &'a str| -> IResult<&'a str, &'a str> {
        let mut paren_depth = 0;
        for (end, ch) in input.char_indices() {
            if (ch == ',' || ch == ')') && paren_depth == 0 {
                return IResult::Ok((&input[end..], &input[..end].trim()));
            }
            if ch == '(' {
                paren_depth += 1;
            } else if ch == ')' {
                paren_depth -= 1;
            }
        }
        IResult::Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Eof,
        )))
    };
    let argument_list = delimited(
        tag("("),
        separated_list0(tag(","), delimited(multispace0, argument, multispace0)),
        tag(")"),
    );
    tuple((identifier, preceded(multispace0, argument_list)))
        .map(|(name, params)| (name, params.into_iter().collect()))
        .parse(input)
}

#[derive(Debug, Clone)]
pub enum Macro {
    Object { body: String },
    Function { params: Vec<String>, body: String },
}
