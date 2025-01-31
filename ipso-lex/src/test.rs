use super::Lexer;
use crate::token::{self, Sign, Token};
use std::rc::Rc;

#[test]
fn lex_char_1() {
    assert_eq!(
        {
            let input = Rc::from("'");
            let lexer = Lexer::new(&input);
            lexer.collect::<Vec<Token>>()
        },
        vec![
            Token {
                data: token::Data::SingleQuote,
                pos: 0,
                column: 0
            },
            Token {
                data: token::Data::Eof,
                pos: 1,
                column: 1
            }
        ]
    )
}

#[test]
fn lex_char_2() {
    assert_eq!(
        {
            let input = Rc::from("'\\");
            let lexer = Lexer::new(&input);
            lexer.collect::<Vec<Token>>()
        },
        vec![
            Token {
                data: token::Data::SingleQuote,
                pos: 0,
                column: 0
            },
            Token {
                data: token::Data::Unexpected('\\'),
                pos: 1,
                column: 1
            },
            Token {
                data: token::Data::Eof,
                pos: 2,
                column: 2
            }
        ]
    )
}

#[test]
fn lex_char_3() {
    assert_eq!(
        {
            let input = Rc::from("'\\\''");
            let lexer = Lexer::new(&input);
            lexer.collect::<Vec<Token>>()
        },
        vec![
            Token {
                data: token::Data::SingleQuote,
                pos: 0,
                column: 0
            },
            Token {
                data: token::Data::Char {
                    value: '\'',
                    length: 2
                },
                pos: 1,
                column: 1
            },
            Token {
                data: token::Data::SingleQuote,
                pos: 3,
                column: 3
            },
            Token {
                data: token::Data::Eof,
                pos: 4,
                column: 4
            }
        ]
    )
}

#[test]
fn lex_int_1() {
    assert_eq!(
        {
            let input = Rc::from("923");
            let lexer = Lexer::new(&input);
            lexer.collect::<Vec<Token>>()
        },
        vec![
            Token {
                data: token::Data::Int {
                    sign: Sign::None,
                    value: 923,
                    length: 3
                },
                pos: 0,
                column: 0
            },
            Token {
                data: token::Data::Eof,
                pos: 3,
                column: 3
            }
        ]
    )
}

#[test]
fn lex_int_2() {
    assert_eq!(
        {
            let input = Rc::from("00923");
            let lexer = Lexer::new(&input);
            lexer.collect::<Vec<Token>>()
        },
        vec![
            Token {
                data: token::Data::Int {
                    sign: Sign::None,
                    value: 923,
                    length: 5
                },
                pos: 0,
                column: 0
            },
            Token {
                data: token::Data::Eof,
                pos: 5,
                column: 5
            }
        ]
    )
}

#[test]
fn lex_import() {
    assert_eq!(
        {
            let input = Rc::from("import yes as no");
            let lexer = Lexer::new(&input);
            lexer.collect::<Vec<Token>>()
        },
        vec![
            Token {
                data: token::Data::Ident(Rc::from("import")),
                pos: 0,
                column: 0,
            },
            Token {
                data: token::Data::Ident(Rc::from("yes")),
                pos: 7,
                column: 7
            },
            Token {
                data: token::Data::Ident(Rc::from("as")),
                pos: 11,
                column: 11
            },
            Token {
                data: token::Data::Ident(Rc::from("no")),
                pos: 14,
                column: 14
            },
            Token {
                data: token::Data::Eof,
                pos: 16,
                column: 16
            }
        ]
    )
}

#[test]
fn lex_definition_1() {
    assert_eq!(
        {
            let input = Rc::from("x : Int\nx = 1");
            let lexer = Lexer::new(&input);
            lexer.collect::<Vec<Token>>()
        },
        vec![
            Token {
                data: token::Data::Ident(Rc::from("x")),
                pos: 0,
                column: 0
            },
            Token {
                data: token::Data::Colon,
                pos: 2,
                column: 2
            },
            Token {
                data: token::Data::Ident(Rc::from("Int")),
                pos: 4,
                column: 4
            },
            Token {
                data: token::Data::Ident(Rc::from("x")),
                pos: 8,
                column: 0
            },
            Token {
                data: token::Data::Equals,
                pos: 10,
                column: 2
            },
            Token {
                data: token::Data::Int {
                    sign: Sign::None,
                    value: 1,
                    length: 1
                },
                pos: 12,
                column: 4
            },
            Token {
                data: token::Data::Eof,
                pos: 13,
                column: 5
            }
        ]
    )
}

#[test]
fn lex_definition_2() {
    assert_eq!(
        {
            let input = Rc::from("x : Int\nx = ~");
            let lexer = Lexer::new(&input);
            lexer.collect::<Vec<Token>>()
        },
        vec![
            Token {
                data: token::Data::Ident(Rc::from("x")),
                pos: 0,
                column: 0
            },
            Token {
                data: token::Data::Colon,
                pos: 2,
                column: 2
            },
            Token {
                data: token::Data::Ident(Rc::from("Int")),
                pos: 4,
                column: 4
            },
            Token {
                data: token::Data::Ident(Rc::from("x")),
                pos: 8,
                column: 0
            },
            Token {
                data: token::Data::Equals,
                pos: 10,
                column: 2
            },
            Token {
                data: token::Data::Unexpected('~'),
                pos: 12,
                column: 4
            },
            Token {
                data: token::Data::Eof,
                pos: 13,
                column: 5
            }
        ]
    )
}

#[test]
fn lex_case_1() {
    assert_eq!(
        {
            let input = Rc::from("case x of\n  a -> b");
            let lexer = Lexer::new(&input);
            lexer.collect::<Vec<Token>>()
        },
        vec![
            Token {
                data: token::Data::Ident(Rc::from("case")),
                pos: 0,
                column: 0
            },
            Token {
                data: token::Data::Ident(Rc::from("x")),
                pos: 5,
                column: 5
            },
            Token {
                data: token::Data::Ident(Rc::from("of")),
                pos: 7,
                column: 7
            },
            Token {
                data: token::Data::Ident(Rc::from("a")),
                pos: 12,
                column: 2
            },
            Token {
                data: token::Data::Arrow,
                pos: 14,
                column: 4
            },
            Token {
                data: token::Data::Ident(Rc::from("b")),
                pos: 17,
                column: 7
            },
            Token {
                data: token::Data::Eof,
                pos: 18,
                column: 8
            }
        ]
    )
}

#[test]
fn lex_ann_1() {
    assert_eq!(
        {
            let input = Rc::from("main : IO ~");
            let lexer = Lexer::new(&input);
            lexer.collect::<Vec<Token>>()
        },
        vec![
            Token {
                data: token::Data::Ident(Rc::from("main")),
                pos: 0,
                column: 0
            },
            Token {
                data: token::Data::Colon,
                pos: 5,
                column: 5
            },
            Token {
                data: token::Data::Ident(Rc::from("IO")),
                pos: 7,
                column: 7
            },
            Token {
                data: token::Data::Unexpected('~'),
                pos: 10,
                column: 10
            },
            Token {
                data: token::Data::Eof,
                pos: 11,
                column: 11
            }
        ]
    )
}

#[test]
fn lex_string_1() {
    let input = Rc::from("\"hello\"");
    let lexer = Lexer::new(&input);
    let expected = vec![
        Token {
            data: token::Data::DoubleQuote,
            pos: 0,
            column: 0,
        },
        Token {
            data: token::Data::String {
                value: String::from("hello"),
                length: 5,
            },
            pos: 1,
            column: 1,
        },
        Token {
            data: token::Data::DoubleQuote,
            pos: 6,
            column: 6,
        },
        Token {
            data: token::Data::Eof,
            pos: 7,
            column: 7,
        },
    ];
    let actual = lexer.collect::<Vec<Token>>();
    assert_eq!(expected, actual)
}

#[test]
fn lex_string_2() {
    let input = Rc::from("\"x $y z\"");
    let lexer = Lexer::new(&input);
    let expected = vec![
        Token {
            data: token::Data::DoubleQuote,
            pos: 0,
            column: 0,
        },
        Token {
            data: token::Data::String {
                value: String::from("x "),
                length: 2,
            },
            pos: 1,
            column: 1,
        },
        Token {
            data: token::Data::Dollar,
            pos: 3,
            column: 3,
        },
        Token {
            data: token::Data::Ident(Rc::from("y")),
            pos: 4,
            column: 4,
        },
        Token {
            data: token::Data::String {
                value: String::from(" z"),
                length: 2,
            },
            pos: 5,
            column: 5,
        },
        Token {
            data: token::Data::DoubleQuote,
            pos: 7,
            column: 7,
        },
        Token {
            data: token::Data::Eof,
            pos: 8,
            column: 8,
        },
    ];
    let actual = lexer.collect::<Vec<Token>>();
    assert_eq!(expected, actual)
}

#[test]
fn lex_string_3() {
    let input = Rc::from("\"x $yy z\"");
    let lexer = Lexer::new(&input);
    let expected = vec![
        Token {
            data: token::Data::DoubleQuote,
            pos: 0,
            column: 0,
        },
        Token {
            data: token::Data::String {
                value: String::from("x "),
                length: 2,
            },
            pos: 1,
            column: 1,
        },
        Token {
            data: token::Data::Dollar,
            pos: 3,
            column: 3,
        },
        Token {
            data: token::Data::Ident(Rc::from("yy")),
            pos: 4,
            column: 4,
        },
        Token {
            data: token::Data::String {
                value: String::from(" z"),
                length: 2,
            },
            pos: 6,
            column: 6,
        },
        Token {
            data: token::Data::DoubleQuote,
            pos: 8,
            column: 8,
        },
        Token {
            data: token::Data::Eof,
            pos: 9,
            column: 9,
        },
    ];
    let actual = lexer.collect::<Vec<Token>>();
    assert_eq!(expected, actual)
}

#[test]
fn lex_string_4() {
    let input = Rc::from("\"x ${yy} z\"");
    let lexer = Lexer::new(&input);
    let expected = vec![
        Token {
            data: token::Data::DoubleQuote,
            pos: 0,
            column: 0,
        },
        Token {
            data: token::Data::String {
                value: String::from("x "),
                length: 2,
            },
            pos: 1,
            column: 1,
        },
        Token {
            data: token::Data::DollarLBrace,
            pos: 3,
            column: 3,
        },
        Token {
            data: token::Data::Ident(Rc::from("yy")),
            pos: 5,
            column: 5,
        },
        Token {
            data: token::Data::RBrace,
            pos: 7,
            column: 7,
        },
        Token {
            data: token::Data::String {
                value: String::from(" z"),
                length: 2,
            },
            pos: 8,
            column: 8,
        },
        Token {
            data: token::Data::DoubleQuote,
            pos: 10,
            column: 10,
        },
        Token {
            data: token::Data::Eof,
            pos: 11,
            column: 11,
        },
    ];
    let actual = lexer.collect::<Vec<Token>>();
    assert_eq!(expected, actual)
}

#[test]
fn lex_string_5() {
    let input = Rc::from("\"x ${a + b} z\"");
    let lexer = Lexer::new(&input);
    let expected = vec![
        Token {
            data: token::Data::DoubleQuote,
            pos: 0,
            column: 0,
        },
        Token {
            data: token::Data::String {
                value: String::from("x "),
                length: 2,
            },
            pos: 1,
            column: 1,
        },
        Token {
            data: token::Data::DollarLBrace,
            pos: 3,
            column: 3,
        },
        Token {
            data: token::Data::Ident(Rc::from("a")),
            pos: 5,
            column: 5,
        },
        Token {
            data: token::Data::Plus,
            pos: 7,
            column: 7,
        },
        Token {
            data: token::Data::Ident(Rc::from("b")),
            pos: 9,
            column: 9,
        },
        Token {
            data: token::Data::RBrace,
            pos: 10,
            column: 10,
        },
        Token {
            data: token::Data::String {
                value: String::from(" z"),
                length: 2,
            },
            pos: 11,
            column: 11,
        },
        Token {
            data: token::Data::DoubleQuote,
            pos: 13,
            column: 13,
        },
        Token {
            data: token::Data::Eof,
            pos: 14,
            column: 14,
        },
    ];
    let actual = lexer.collect::<Vec<Token>>();
    assert_eq!(expected, actual)
}

#[test]
fn lex_string_6() {
    let input = Rc::from("\"hello $name\"");
    let lexer = Lexer::new(&input);
    let expected = vec![
        Token {
            data: token::Data::DoubleQuote,
            pos: 0,
            column: 0,
        },
        Token {
            data: token::Data::String {
                value: String::from("hello "),
                length: 6,
            },
            pos: 1,
            column: 1,
        },
        Token {
            data: token::Data::Dollar,
            pos: 7,
            column: 7,
        },
        Token {
            data: token::Data::Ident(Rc::from("name")),
            pos: 8,
            column: 8,
        },
        Token {
            data: token::Data::DoubleQuote,
            pos: 12,
            column: 12,
        },
        Token {
            data: token::Data::Eof,
            pos: 13,
            column: 13,
        },
    ];
    let actual = lexer.collect::<Vec<Token>>();
    assert_eq!(expected, actual)
}

#[test]
fn lex_string_7() {
    let input = Rc::from("\"hello\\n\\n\\nworld\"");
    let lexer = Lexer::new(&input);
    let expected = vec![
        Token {
            data: token::Data::DoubleQuote,
            pos: 0,
            column: 0,
        },
        Token {
            data: token::Data::String {
                value: String::from("hello\n\n\nworld"),
                length: 16,
            },
            pos: 1,
            column: 1,
        },
        Token {
            data: token::Data::DoubleQuote,
            pos: 17,
            column: 17,
        },
        Token {
            data: token::Data::Eof,
            pos: 18,
            column: 18,
        },
    ];
    let actual = lexer.collect::<Vec<Token>>();
    assert_eq!(expected, actual)
}

#[test]
fn lex_cmd_1() {
    let input = Rc::from("`  ls  -laR   `");
    let lexer = Lexer::new(&input);
    let expected = vec![
        Token {
            data: token::Data::Backtick,
            pos: 0,
            column: 0,
        },
        Token {
            data: token::Data::Cmd(Rc::from("ls")),
            pos: 3,
            column: 3,
        },
        Token {
            data: token::Data::Cmd(Rc::from("-laR")),
            pos: 7,
            column: 7,
        },
        Token {
            data: token::Data::Backtick,
            pos: 14,
            column: 14,
        },
        Token {
            data: token::Data::Eof,
            pos: 15,
            column: 15,
        },
    ];
    let actual = lexer.collect::<Vec<Token>>();
    assert_eq!(expected, actual)
}

#[test]
fn lex_cmd_2() {
    let input = Rc::from("`echo \"hello\"`");
    let lexer = Lexer::new(&input);
    let expected = vec![
        Token {
            data: token::Data::Backtick,
            pos: 0,
            column: 0,
        },
        Token {
            data: token::Data::Cmd(Rc::from("echo")),
            pos: 1,
            column: 1,
        },
        Token {
            data: token::Data::DoubleQuote,
            pos: 6,
            column: 6,
        },
        Token {
            data: token::Data::String {
                value: String::from("hello"),
                length: 5,
            },
            pos: 7,
            column: 7,
        },
        Token {
            data: token::Data::DoubleQuote,
            pos: 12,
            column: 12,
        },
        Token {
            data: token::Data::Backtick,
            pos: 13,
            column: 13,
        },
        Token {
            data: token::Data::Eof,
            pos: 14,
            column: 14,
        },
    ];
    let actual = lexer.collect::<Vec<Token>>();
    assert_eq!(expected, actual)
}

#[test]
fn lex_cmd_3() {
    let input = Rc::from("`echo $a`");
    let lexer = Lexer::new(&input);
    let expected = vec![
        Token {
            data: token::Data::Backtick,
            pos: 0,
            column: 0,
        },
        Token {
            data: token::Data::Cmd(Rc::from("echo")),
            pos: 1,
            column: 1,
        },
        Token {
            data: token::Data::Dollar,
            pos: 6,
            column: 6,
        },
        Token {
            data: token::Data::Ident(Rc::from("a")),
            pos: 7,
            column: 7,
        },
        Token {
            data: token::Data::Backtick,
            pos: 8,
            column: 8,
        },
        Token {
            data: token::Data::Eof,
            pos: 9,
            column: 9,
        },
    ];
    let actual = lexer.collect::<Vec<Token>>();
    assert_eq!(expected, actual)
}
