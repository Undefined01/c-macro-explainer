use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{alpha1, alphanumeric1, multispace0, multispace1},
    combinator::{map, opt, recognize, value},
    multi::{many0_count, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};
use std::{
    ascii::escape_default, cell::RefCell, collections::{HashMap, HashSet}
};

use crate::parser::{
    identifier, parse_comment, parse_concatenation, parse_define, parse_macro_call,
    parse_stringify, parse_undef, Macro,
};

#[derive(Debug, Default)]
struct MacroPreprocessor {
    macros: HashMap<String, Macro>,
}

impl MacroPreprocessor {
    fn new() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    fn define_macro(&mut self, name: String, definition: Macro) {
        self.macros.insert(name, definition);
    }

    fn undefine_macro(&mut self, name: &str) {
        self.macros.remove(name);
    }

    fn expand_object_macro(&self, expended_macros: &mut HashSet<String>, name: &str) -> Option<String> {
        if expended_macros.contains(name) {
            return None;
        }
        if let Some(Macro::Object { body }) = self.macros.get(name) {
            expended_macros.insert(name.to_string());
            Some(body.clone())
        } else {
            None
        }
    }

    fn get_argument(params: &Vec<String>, args: &Vec<String>, arg_name: &str) -> String {
        if arg_name == "__VA_ARGS__" {
            args.iter().skip(params.len() - 1).join(", ")
        } else {
            params
                .iter()
                .position(|p| p == arg_name)
                .map(|i| args[i].to_string())
                .unwrap_or(arg_name.to_string())
        }
    }

    fn expand_function_macro(&self, expended_macros: &mut HashSet<String>, macro_name: &str, macro_args: Vec<&str>) -> Option<String> {
        if expended_macros.contains(macro_name) {
            return None;
        }
        if let Some(Macro::Function { params, body }) = self.macros.get(macro_name) {
            let args = if body.contains("##") {
                macro_args.iter().map(|s| s.to_string()).collect()
            } else {
                macro_args.iter().map(|s| self.process(s)).collect()
            };
            eprintln!("pre-expanded: {:?}", args);

            expended_macros.insert(macro_name.to_string());
            let mut result = String::new();
            let mut input = body.as_str();
            while !input.is_empty() {
                if let Ok((remaining, name)) = parse_stringify(input) {
                    let arg = Self::get_argument(params, &args, name);
                    result.push_str(
                        &String::from_utf8(arg.bytes().flat_map(escape_default).collect()).unwrap(),
                    );
                    input = remaining;
                    continue;
                }
                if let Ok((remaining, (lhs, rhs))) = parse_concatenation(input) {
                    let larg = Self::get_argument(params, &args, lhs);
                    let rarg = Self::get_argument(params, &args, rhs);
                    result.push_str(&larg);
                    result.push_str(&rarg);
                    input = remaining;
                    continue;
                }
                if let Ok((remaining, name)) = identifier(input) {
                    let arg = Self::get_argument(params, &args, name);
                    result.push_str(&arg);
                    eprintln!("arg {} -> {}", name, arg);
                    input = remaining;
                    continue;
                }

                result.push_str(&input[..1]);
                input = &input[1..];
            }
            Some(result)
        } else {
            None
        }
    }

    fn process(&self, input: &str) -> String {
        let mut input = input.to_string();
        let mut result = String::with_capacity(input.len());
        let mut macro_generated_pos = 0i32;
        let ref mut expended_macros = HashSet::new();

        while !input.is_empty() {
            if let Ok((remaining, _)) = parse_comment(&input) {
                let eatten = input.len() - remaining.len();
                input.replace_range(..eatten, "");
                macro_generated_pos -= eatten as i32;
                if macro_generated_pos < 0 {
                    macro_generated_pos = 0;
                    expended_macros.clear();
                }
            } else if let Ok((remaining, (name, args))) = parse_macro_call(&input) {
                if let Some(expansion) = self.expand_function_macro(expended_macros, name, args) {
                    let eatten = input.len() - remaining.len();
                    input.replace_range(..eatten, &expansion);
                    macro_generated_pos -= eatten as i32;
                    if macro_generated_pos < 0 {
                        macro_generated_pos = 0;
                    }
                    macro_generated_pos += expansion.len() as i32;
                } else {
                    let eatten = input.len() - identifier(&input).unwrap().0.len();
                    result.push_str(&input[..eatten]);
                    input.replace_range(..eatten, "");
                    macro_generated_pos -= eatten as i32;
                    if macro_generated_pos < 0 {
                        macro_generated_pos = 0;
                        expended_macros.clear();
                    }
                }
            } else if let Ok((remaining, name)) = identifier(&input) {
                if let Some(expansion) = self.expand_object_macro(expended_macros, name) {
                    let eatten = input.len() - remaining.len();
                    input.replace_range(..eatten, &expansion);
                    macro_generated_pos -= eatten as i32;
                    if macro_generated_pos < 0 {
                        macro_generated_pos = 0;
                    }
                    macro_generated_pos += expansion.len() as i32;
                } else {
                    let eatten = input.len() - remaining.len();
                    result.push_str(&input[..eatten]);
                    input.replace_range(..eatten, "");
                    macro_generated_pos -= eatten as i32;
                    if macro_generated_pos < 0 {
                        macro_generated_pos = 0;
                        expended_macros.clear();
                    }
                }
            } else {
                let eatten = 1;
                result.push_str(&input[..eatten]);
                input.replace_range(..eatten, "");
                macro_generated_pos -= eatten as i32;
                if macro_generated_pos < 0 {
                    macro_generated_pos = 0;
                    expended_macros.clear();
                }
            }
        }

        result
    }

    fn process_mut(&mut self, input: &str) -> String {
        let mut input = input.to_string();
        let mut result = String::with_capacity(input.len());
        let mut macro_generated_pos = 0i32;
        let ref mut expended_macros = HashSet::new();

        while !input.is_empty() {
            if let Ok((remaining, _)) = parse_comment(&input) {
                let eatten = input.len() - remaining.len();
                input.replace_range(..eatten, "");
                macro_generated_pos -= eatten as i32;
                if macro_generated_pos < 0 {
                    macro_generated_pos = 0;
                    expended_macros.clear();
                }
            } else if let Ok((remaining, name)) = parse_undef(&input) {
                self.undefine_macro(name);
                let eatten = input.len() - remaining.len();
                input.replace_range(..eatten + 1, ""); // the new line
                macro_generated_pos -= eatten as i32;
                if macro_generated_pos < 0 {
                    macro_generated_pos = 0;
                    expended_macros.clear();
                }
            } else if let Ok((remaining, (name, macro_def))) = parse_define(&input) {
                self.define_macro(name.to_string(), macro_def);
                let eatten = input.len() - remaining.len();
                input.replace_range(..eatten + 1, ""); // the new line
                macro_generated_pos -= eatten as i32;
                if macro_generated_pos < 0 {
                    macro_generated_pos = 0;
                    expended_macros.clear();
                }
            } else if let Ok((remaining, (name, args))) = parse_macro_call(&input) {
                if let Some(expansion) = self.expand_function_macro(expended_macros, name, args) {
                    // result.push_str(&expansion);
                    let eatten = input.len() - remaining.len();
                    input.replace_range(..eatten, &expansion);
                    macro_generated_pos -= eatten as i32;
                    if macro_generated_pos < 0 {
                        macro_generated_pos = 0;
                    }
                    macro_generated_pos += expansion.len() as i32;
                } else {
                    let eatten = input.len() - identifier(&input).unwrap().0.len();
                    result.push_str(&input[..eatten]);
                    input.replace_range(..eatten, "");
                    macro_generated_pos -= eatten as i32;
                    if macro_generated_pos < 0 {
                        macro_generated_pos = 0;
                        expended_macros.clear();
                    }
                }
            } else if let Ok((remaining, name)) = identifier(&input) {
                if let Some(expansion) = self.expand_object_macro(expended_macros, name) {
                    // result.push_str(&expansion);
                    let eatten = input.len() - remaining.len();
                    input.replace_range(..eatten, &expansion);
                    macro_generated_pos -= eatten as i32;
                    if macro_generated_pos < 0 {
                        macro_generated_pos = 0;
                    }
                    macro_generated_pos += expansion.len() as i32;
                } else {
                    let eatten = input.len() - remaining.len();
                    result.push_str(&input[..eatten]);
                    input.replace_range(..eatten, "");
                    macro_generated_pos -= eatten as i32;
                    if macro_generated_pos < 0 {
                        macro_generated_pos = 0;
                        expended_macros.clear();
                    }
                }
            } else {
                let eatten = 1;
                result.push_str(&input[..eatten]);
                input.replace_range(..eatten, "");
                macro_generated_pos -= eatten as i32;
                if macro_generated_pos < 0 {
                    macro_generated_pos = 0;
                    expended_macros.clear();
                }
            }
        }

        result
    }
}

fn preprocess(input: &str) -> String {
    let mut preprocessor = MacroPreprocessor::new();
    preprocessor.process_mut(input)
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::process::Stdio;

    use super::*;

    fn get_reference_result(source_code: &str) -> Option<String> {
        std::env::set_var("OPT_LEVEL", "0");
        std::env::set_var("TARGET", "x86_64-unknown-linux-gnu");
        std::env::set_var("HOST", "x86_64-unknown-linux-gnu");

        let tool = cc::Build::new().get_compiler();
        let mut command = tool.to_command();
        command.arg("-E").arg("-P").arg("-");
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(source_code.as_bytes()).unwrap();
        stdin.flush().unwrap();
        drop(stdin);
        let result = child.wait_with_output().unwrap();
        if result.status.success() {
            let expanded_code = String::from_utf8(result.stdout).ok()?;
            Some(expanded_code)
        } else {
            None
        }
    }

    fn standardize(mut s: String) -> String {
        loop {
            let mut new = s.replace("\n\n", "\n");
            new = new.replace(" \n", "\n");
            if new.len() == s.len() {
                break;
            }
            s = new;
        }
        s.trim().to_string()
    }

    fn test(source_code: &str) {
        let expanded_code = preprocess(source_code);
        let expanded_code = standardize(expanded_code);
        let reference = get_reference_result(source_code).unwrap();
        let reference = standardize(reference);
        assert_eq!(
            expanded_code, reference,
            "Source Code:\n{}\n\nExpanded Code:\n{}\n\nReference:\n{}",
            source_code, expanded_code, reference,
        );
    }

    #[test]
    fn simple_replacing() {
        test(
            r#"
#define PI 3.14
#define EMPTY()
#define IDENTICAL(x) x
#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define VALUE(x, xx, xxx) x xx xxx

double radius = 5;
double area = PI * radius * radius;
EMPTY();
IDENTICAL(11);
VALUE(1, 2, 3);
int max_value = IDENTICAL(MAX)(10, 20);
"#,
        );
    }

    #[test]
    fn concatenation() {
        test(
            r#"
#define IIF(cond) IIF_ ## cond
#define IIF_0(t, f) f
#define IIF_1(t, f) t

#define A() 1
//This expands to true
IIF(1)(true, false)
// This expands to IIF_A()(true, false) because A() doesn't expand to 1.
// Further expansion is inhibited by the ## operator
IIF(A())(true, false)
"#,
        );

        test(
            r#"
#define PRIMITIVE_CAT(a, ...) a ## __VA_ARGS__
#define IIF(c) PRIMITIVE_CAT(IIF_, c)
#define IIF_0(t, ...) __VA_ARGS__
#define IIF_1(t, ...) t

#define A() 1
//This expands to true
IIF(1)(true, false)
// This expands to true because IIF contains no ## operator and A() get expanded to 1 before expanding PRIOMITIVE_CAT
IIF(A())(true, false)
"#,
        );

        test(
            r#"
#define CHECK_N(x, n, ...) n
#define CHECK(...) CHECK_N(__VA_ARGS__, 0,)
#define PROBE(x) x, 1,

// Expands to 0
CHECK(xxx)
// Expands to 1
// PROBE produce two arguments, making the second argument of CHECK_N being 1 instead of 0
CHECK(PROBE(~))

#define IS_PAREN(x) CHECK(IS_PAREN_PROBE x)
#define IS_PAREN_PROBE(...) PROBE(~)
IS_PAREN(())
IS_PAREN(xxx)

#define NOT(x) CHECK(PRIMITIVE_CAT(NOT_, x))
#define NOT_0 PROBE(~)
NOT(0)
NOT(1)
NOT(xxx)

#define PRIMITIVE_CAT(a, ...) a ## __VA_ARGS__
#define IIF(c) PRIMITIVE_CAT(IIF_, c)
#define IIF_0(t, ...) __VA_ARGS__
#define IIF_1(t, ...) t
#define BOOL(x) NOT(NOT(x))
#define IF(c) IIF(BOOL(c))
IF(0)(true, false)
IF(1)(true, false)
IF(xxx)(true, false)

#define EAT(...)
#define EXPAND(...) __VA_ARGS__
#define WHEN(c) IF(c)(EXPAND, EAT)
WHEN(0)(xxx)
WHEN(1)(xxx)
WHEN(xxx)(xxx)
"#,
        );
    }

    #[test]
    fn recursion() {
        test(
            r#"
#define EMPTY()
#define DEFER(id) id EMPTY()
#define EXPAND(...) __VA_ARGS__

#define A() 123
A() // Expands to 123
//  DEFER(A)() => A EMPTY()() => A ()
// |^^^^^^^^       |^^^^^^^       |
// `^` indicates the current redex, and `|` indicates the boundary of the processed symbols.
// Though the expansion of EMPTY produces a new redex, A is already marked as processed and will not be expanded again.
DEFER(A)()
// EXPAND(DEFER(A)()) => EXPAND(A EMPTY()()) => EXPAND(A ()) => EXPAND(A ()) => A () => 123
//       |^^^^^^^^               |^^^^^^                |      |^^^^^^^^^^^^   |^^^^   |
// Note that the preprocessor will do the pre-expansion for EXPAND first, which expands the argument to `A ()`.
// And after the expansion of EXPAND, `A ()` is expanded again.
EXPAND(DEFER(A)())

#define PRIMITIVE_CAT(a, ...) a ## __VA_ARGS__
#define IIF(c) PRIMITIVE_CAT(IIF_, c)
#define IIF_0(t, ...) __VA_ARGS__
#define IIF_1(t, ...) t
#define CHECK_N(x, n, ...) n
#define CHECK(...) CHECK_N(__VA_ARGS__, 0,)
#define PROBE(x) x, 1,
#define NOT(x) CHECK(PRIMITIVE_CAT(NOT_, x))
#define NOT_0 PROBE(~)
#define BOOL(x) NOT(NOT(x))
#define IF(c) IIF(BOOL(c))
#define EAT(...)
#define WHEN(c) IF(c)(EXPAND, EAT)
#define DEC(x) PRIMITIVE_CAT(DEC_, x)
#define DEC_0 0
#define DEC_1 0
#define DEC_2 1
#define DEC_3 2
#define DEC_4 3
#define DEC_5 4
#define DEC_6 5
#define DEC_7 6
#define DEC_8 7
#define DEC_9 8
#define EVAL(...)  EVAL1(EVAL1(EVAL1(__VA_ARGS__)))
#define EVAL1(...) EVAL2(EVAL2(EVAL2(__VA_ARGS__)))
#define EVAL2(...) EVAL3(EVAL3(EVAL3(__VA_ARGS__)))
#define EVAL3(...) EVAL4(EVAL4(EVAL4(__VA_ARGS__)))
#define EVAL4(...) EVAL5(EVAL5(EVAL5(__VA_ARGS__)))
#define EVAL5(...) __VA_ARGS__
#define OBSTRUCT(...) __VA_ARGS__ DEFER(EMPTY)()
#define REPEAT(count, macro, ...) \
    WHEN(count) \
    ( \
        OBSTRUCT(REPEAT_INDIRECT) () \
        ( \
            DEC(count), macro, __VA_ARGS__ \
        ) \
        OBSTRUCT(macro) \
        ( \
            DEC(count), __VA_ARGS__ \
        ) \
    )
#define REPEAT_INDIRECT() REPEAT
#define M(i, _) i
EVAL(REPEAT(8, M, ~)) // 0 1 2 3 4 5 6 7
"#,
        );
    }

    #[test]
    fn comparision() {
        test(
            r#"
#define PRIMITIVE_CAT(a, ...) a ## __VA_ARGS__
#define IIF(c) PRIMITIVE_CAT(IIF_, c)
#define IIF_0(t, ...) __VA_ARGS__
#define IIF_1(t, ...) t
#define CHECK_N(x, n, ...) n
#define CHECK(...) CHECK_N(__VA_ARGS__, 0,)
#define PROBE(x) x, 1,
#define IS_PAREN(x) CHECK(IS_PAREN_PROBE x)
#define IS_PAREN_PROBE(...) PROBE(~)
#define COMPARE_foo(x) x
#define COMPARE_bar(x) x
#define PRIMITIVE_COMPARE(x, y) IS_PAREN \
( \
COMPARE_ ## x ( COMPARE_ ## y) (())  \
)

// COMPARE_foo ( COMPARE_foo ) (()) => COMPARE_foo (())
// Note that the second expansion of COMPARE_foo is prevented by the [self-reference rule](https://gcc.gnu.org/onlinedocs/gcc-14.2.0/cpp/Self-Referential-Macros.html)
COMPARE_foo ( COMPARE_foo ) (())
// COMPARE_foo ( COMPARE_bar ) (()) => COMPARE_bar (()) => ()
COMPARE_foo ( COMPARE_bar ) (())
PRIMITIVE_COMPARE(foo, bar) // Expands to 1
PRIMITIVE_COMPARE(foo, foo) // Expands to 0
PRIMITIVE_COMPARE(foo, unfoo)

#define CAT(a, ...) PRIMITIVE_CAT(a, __VA_ARGS__)
#define EAT(...)
#define COMPL(b) PRIMITIVE_CAT(COMPL_, b)
#define COMPL_0 1
#define COMPL_1 0
#define BITAND(x) PRIMITIVE_CAT(BITAND_, x)
#define BITAND_0(y) 0
#define BITAND_1(y) y
#define IS_COMPARABLE(x) IS_PAREN( CAT(COMPARE_, x) (()) )
#define NOT_EQUAL(x, y) \
IIF(BITAND(IS_COMPARABLE(x))(IS_COMPARABLE(y)) ) \
( \
   PRIMITIVE_COMPARE, \
   1 EAT \
)(x, y)
#define EQUAL(x, y) COMPL(NOT_EQUAL(x, y))
EQUAL(foo, bar) // Expands to 0
EQUAL(foo, foo) // Expands to 1
EQUAL(foo, unfoo) // Expands to 0
EQUAL(unfoo, unfoo) // Expands to 0
"#,
        );
    }

    #[test]
    fn self_reference() {
        test(
            r#"
#define EPERM EPERM
EPERM

#define foo (4 + foo)
foo

#define x (4 + y)
#define y (2 * x)
x
y
"#,
        );
    }

    #[test]
    fn prescan() {
        test(
            r#"
#define f(x) ((x) + 1)
f(1)
f(f(1))
"#,
        );

        test(
            r#"
#define AFTERX(x) X_ ## x
#define XAFTERX(x) AFTERX(x)
#define TABLESIZE 1024
#define BUFSIZE TABLESIZE
AFTERX(BUFSIZE)
XAFTERX(BUFSIZE)
"#,
        );
    }
}
