# c_macro_explainer

A command-line tool to provide a step-by-step explanation of how C-like macros expand and evaluate.

Have you ever wondered how the following macro can check whether a macro is defined and set to true? Or how specific macros expand? This tool helps you understand these expansions in a clear and detailed manner.

```c
IF(0)(true, false)      // expands to false
IF(1)(true, false)      // expands to true
#define TRUE 1
IF(TRUE)(true, false)   // expands to true
IF(xxx)(true, false)    // xxx is undefined, it expands to false
```

### Features

- Explains the expansion of C-like macros step by step.
- Helps in debugging and understanding the behavior of macros in C/C++ code.

This tool is not guaranteed to be fully compatible with GCC or Clang preprocessing rules. It does not support other preprocessor directives like #include, #if, #ifdef, etc. It is a simplified version to help understand the basics of macro expansion. If you find any discrepancies, please open an issue.

### Usage

Use `cargo run` under the project directory to run the tool,
or build the project using `cargo build --release` and run the executable from the target directory.

The tool reads input from the standard input and provides a detailed explanation of the macro expansion.

```sh
$ ./c_macro_explainer
#define CHECK_N(x, n, ...) n
#define CHECK(...) CHECK_N(__VA_ARGS__, 0,)
#define PROBE(x) x, 1,

// Expands to 0
CHECK(xxx)
// Expands to 1
CHECK(PROBE(~))
```

To end the input, press Ctrl+D (Linux/macOS) or Ctrl+Z (Windows). The example above will output the following:

```
Expanding function-like macro CHECK with args {__VA_ARGS__=>`xxx`}. The result is `CHECK_N(xxx, 0,)`
Expanding function-like macro CHECK_N with args {x=>`xxx`, n=>`0`, __VA_ARGS__=>``}. The result is `0`
Expanding function-like macro PROBE with args {x=>`~`}. The result is `~, 1,`
Expanding function-like macro CHECK with args {__VA_ARGS__=>`~, 1,`}. The result is `CHECK_N(~, 1,, 0,)`
Expanding function-like macro CHECK_N with args {x=>`~`, n=>`1`, __VA_ARGS__=>`, 0, `}. The result is `1`
```

The explaination of the example in this project's introduction is a bit long.

<details>
<summary>Source Code</summary>

```c
#define CHECK_N(x, n, ...) n
#define CHECK(...) CHECK_N(__VA_ARGS__, 0,)
#define PROBE(x) x, 1,

// Expands to 0
CHECK(xxx)
// Expands to 1
// PROBE produce two arguments, making the second argument of CHECK_N being 1 instead of 0
CHECK(PROBE(~))

#define PRIMITIVE_CAT(a, ...) a ## __VA_ARGS__
#define IIF(c) PRIMITIVE_CAT(IIF_, c)
#define IIF_0(t, ...) __VA_ARGS__
#define IIF_1(t, ...) t
#define NOT(x) CHECK(PRIMITIVE_CAT(NOT_, x))
#define NOT_1 PROBE(~)
#define BOOL(x) NOT(x)
#define IF(c) IIF(BOOL(c))

// Expands to false
IF(0)(true, false)
// Expands to true
IF(1)(true, false)
#define TRUE 1
// Expands to true
IF(TRUE)(true, false)
// Expands to false
IF(xxx)(true, false)
```
</details>

<details>
<summary>Explainations</summary>

```
Expanding function-like macro CHECK with args {__VA_ARGS__=>`xxx`}. The result is `CHECK_N(xxx, 0,)`
Expanding function-like macro CHECK_N with args {x=>`xxx`, n=>`0`, __VA_ARGS__=>``}. The result is `0`
Expanding function-like macro PROBE with args {x=>`~`}. The result is `~, 1,`
Expanding function-like macro CHECK with args {__VA_ARGS__=>`~, 1,`}. The result is `CHECK_N(~, 1,, 0,)`
Expanding function-like macro CHECK_N with args {x=>`~`, n=>`1`, __VA_ARGS__=>`, 0, `}. The result is `1`
Expanding function-like macro IF with args {c=>`0`}. The result is `IIF(BOOL(0))`
Expanding function-like macro BOOL with args {x=>`0`}. The result is `NOT(0)`
Expanding function-like macro NOT with args {x=>`0`}. The result is `CHECK(PRIMITIVE_CAT(NOT_, 0))`
The body of PRIMITIVE_CAT contains ##, so the pre-expansion is skipped
Expanding function-like macro PRIMITIVE_CAT with args {a=>`NOT_`, __VA_ARGS__=>`0`}. The result is `NOT_0`
Expanding function-like macro CHECK with args {__VA_ARGS__=>`NOT_0`}. The result is `CHECK_N(NOT_0, 0,)`
Expanding function-like macro CHECK_N with args {x=>`NOT_0`, n=>`0`, __VA_ARGS__=>``}. The result is `0`
Expanding function-like macro IIF with args {c=>`0`}. The result is `PRIMITIVE_CAT(IIF_, 0)`
The body of PRIMITIVE_CAT contains ##, so the pre-expansion is skipped
Expanding function-like macro PRIMITIVE_CAT with args {a=>`IIF_`, __VA_ARGS__=>`0`}. The result is `IIF_0`
Expanding function-like macro IIF_0 with args {t=>`true`, __VA_ARGS__=>`false`}. The result is `false`
Expanding function-like macro IF with args {c=>`1`}. The result is `IIF(BOOL(1))`
Expanding function-like macro BOOL with args {x=>`1`}. The result is `NOT(1)`
Expanding function-like macro NOT with args {x=>`1`}. The result is `CHECK(PRIMITIVE_CAT(NOT_, 1))`
The body of PRIMITIVE_CAT contains ##, so the pre-expansion is skipped
Expanding function-like macro PRIMITIVE_CAT with args {a=>`NOT_`, __VA_ARGS__=>`1`}. The result is `NOT_1`
Expanding object-like macro NOT_1 to `PROBE(~)`
Expanding function-like macro PROBE with args {x=>`~`}. The result is `~, 1,`
Expanding function-like macro CHECK with args {__VA_ARGS__=>`~, 1,`}. The result is `CHECK_N(~, 1,, 0,)`
Expanding function-like macro CHECK_N with args {x=>`~`, n=>`1`, __VA_ARGS__=>`, 0, `}. The result is `1`
Expanding function-like macro IIF with args {c=>`1`}. The result is `PRIMITIVE_CAT(IIF_, 1)`
The body of PRIMITIVE_CAT contains ##, so the pre-expansion is skipped
Expanding function-like macro PRIMITIVE_CAT with args {a=>`IIF_`, __VA_ARGS__=>`1`}. The result is `IIF_1`
Expanding function-like macro IIF_1 with args {t=>`true`, __VA_ARGS__=>`false`}. The result is `true`
Expanding object-like macro TRUE to `1`
Expanding function-like macro IF with args {c=>`1`}. The result is `IIF(BOOL(1))`
Expanding function-like macro BOOL with args {x=>`1`}. The result is `NOT(1)`
Expanding function-like macro NOT with args {x=>`1`}. The result is `CHECK(PRIMITIVE_CAT(NOT_, 1))`
The body of PRIMITIVE_CAT contains ##, so the pre-expansion is skipped
Expanding function-like macro PRIMITIVE_CAT with args {a=>`NOT_`, __VA_ARGS__=>`1`}. The result is `NOT_1`
Expanding object-like macro NOT_1 to `PROBE(~)`
Expanding function-like macro PROBE with args {x=>`~`}. The result is `~, 1,`
Expanding function-like macro CHECK with args {__VA_ARGS__=>`~, 1,`}. The result is `CHECK_N(~, 1,, 0,)`
Expanding function-like macro CHECK_N with args {x=>`~`, n=>`1`, __VA_ARGS__=>`, 0, `}. The result is `1`
Expanding function-like macro IIF with args {c=>`1`}. The result is `PRIMITIVE_CAT(IIF_, 1)`
The body of PRIMITIVE_CAT contains ##, so the pre-expansion is skipped
Expanding function-like macro PRIMITIVE_CAT with args {a=>`IIF_`, __VA_ARGS__=>`1`}. The result is `IIF_1`
Expanding function-like macro IIF_1 with args {t=>`true`, __VA_ARGS__=>`false`}. The result is `true`
Expanding function-like macro IF with args {c=>`xxx`}. The result is `IIF(BOOL(xxx))`
Expanding function-like macro BOOL with args {x=>`xxx`}. The result is `NOT(xxx)`
Expanding function-like macro NOT with args {x=>`xxx`}. The result is `CHECK(PRIMITIVE_CAT(NOT_, xxx))`
The body of PRIMITIVE_CAT contains ##, so the pre-expansion is skipped
Expanding function-like macro PRIMITIVE_CAT with args {a=>`NOT_`, __VA_ARGS__=>`xxx`}. The result is `NOT_xxx`
Expanding function-like macro CHECK with args {__VA_ARGS__=>`NOT_xxx`}. The result is `CHECK_N(NOT_xxx, 0,)`
Expanding function-like macro CHECK_N with args {x=>`NOT_xxx`, n=>`0`, __VA_ARGS__=>``}. The result is `0`
Expanding function-like macro IIF with args {c=>`0`}. The result is `PRIMITIVE_CAT(IIF_, 0)`
The body of PRIMITIVE_CAT contains ##, so the pre-expansion is skipped
Expanding function-like macro PRIMITIVE_CAT with args {a=>`IIF_`, __VA_ARGS__=>`0`}. The result is `IIF_0`
Expanding function-like macro IIF_0 with args {t=>`true`, __VA_ARGS__=>`false`}. The result is `false`
Preprocessed code:
0
1
false
true
true
false
```
</details>


### Explanation of C-like Macros

Here are some brief explanations of the macros expansion rules.

- **Object-like Macros**: These macros are simple text substitutions. They are defined using `#define` and do not take any arguments.
- **Function-like Macros**: These macros can take arguments and are defined using `#define`. They are expanded by replacing the macro name with the macro body and substituting the arguments.
- **Argument Prescan**: Before expanding a macro, the arguments are scanned for other macros to expand. This process is called argument prescan. For example, `F(G(x))` will first expand `G(x)` before expanding `F`.
    <details>
    <summary>Try it</summary>

    ```c
    #define f(x) #x
    #define g(x) ((x) * 2)
    // Expands to "((1) * 2)" instead of "g(1)"
    f(g(1))
    ```
    </details>


- **Concatenation**: The `##` operator concatenates two tokens into a single token. But the concatenated tokens are not expanded before concatenation. For example, `A ## B` will be `AB`, not `XY` if `A` is defined as `X` and `B` is defined as `Y`. To bypass this behavior, wrap the concatenated tokens in another macro to apply argument prescan to expand them before concatenation.
    <details>
    <summary>Try it</summary>

    ```c
    #define PRIMITIVE_CONCAT(x, ...) x ## __VA_ARGS__
    #define CONCAT(x, ...) PRIMITIVE_CONCAT(x, __VA_ARGS__)
    #define A xxx
    #define B yyy
    PRIMITIVE_CONCAT(A, B)      // AB
    PRIMITIVE_CONCAT(A, B, A)   // AB, xxx
    CONCAT(A, B)                // xxxyyy
    CONCAT(A, B, A)             // xxxyyy, xxx
    ```
    </details>

- **Rescanning and Further Replacement**: After the argument prescan and the macro expansion, the resulting text is scanned again for further macro replacement.
    <details>
    <summary>Try it</summary>

    ```c
    #define PRIMITIVE_CONCAT(x, ...) x ## __VA_ARGS__
    #define CONCAT(x, ...) PRIMITIVE_CONCAT(x, __VA_ARGS__)
    #define A xxx
    #define B yyy
    #define AB ab
    PRIMITIVE_CONCAT(A, B)      // ab
    PRIMITIVE_CONCAT(A, B, A)   // ab, xxx
    CONCAT(A, B)                // xxxyyy
    CONCAT(A, B, A)             // xxxyyy, xxx
    ```
    </details>

- **Self-Referential Macros**: Macros can refer to themselves in their definitions. This can lead to infinite recursion if not handled properly. So the same macro is expanded only once in self-referential macros. For example, `#define A A` will not cause infinite recursion. This also applies to mutually recursive macros like `#define A B` and `#define B A`.

    Note that the self-reference from the arguments are also applicable to this rule. For example, `#define A(x) x` and `A(A)` will not cause infinite recursion.

    <details>
    <summary>Try it</summary>

    ```c
    #define EPERM EPERM
    EPERM   // EPERM

    #define foo (4 + foo)
    foo     // (4 + foo)

    #define x (4 + y)
    #define y (2 * x)
    x       // (4 + (2 * x))
    y       // (2 * (4 + y))

    #define A(x) x
    A(A)    // A
    ```
    </details>


### Acknowledgments

Thanks to [cloak](https://github.com/pfultz2/Cloak/wiki/C-Preprocessor-tricks,-tips,-and-idioms) for the inspiration and the C preprocessor tricks.