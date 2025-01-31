#[cfg(test)]
mod test;

pub mod grammar;
pub mod indentation;
pub mod operator;

use fixedbitset::FixedBitSet;
use fnv::FnvHashSet;
use ipso_diagnostic::{Diagnostic, Location, Message, Source};
use ipso_lex::{
    token::{self, Relation, Sign, Token},
    Lexer,
};
use ipso_syntax::{self as syntax, Binop, Keyword, Module, Spanned};
use std::{
    collections::BTreeSet,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    rc::Rc,
    vec,
};

use crate::grammar::module::module;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Unexpected {
        source: Source,
        pos: usize,
        expecting: BTreeSet<token::Name>,
    },
    AmbiguousUseOf {
        source: Source,
        pos: usize,
        operator: Binop,
    },
}

impl Error {
    fn source(&self) -> Source {
        match self {
            Error::Unexpected { source, .. } => source.clone(),
            Error::AmbiguousUseOf { source, .. } => source.clone(),
        }
    }

    pub fn position(&self) -> usize {
        match self {
            Error::Unexpected { pos, .. } => *pos,
            Error::AmbiguousUseOf { pos, .. } => *pos,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Error::Unexpected { expecting, .. } => {
                let mut str = String::from("expected one of: ");
                let mut iter = expecting.iter();
                match iter.next() {
                    None => str,
                    Some(token) => {
                        str.push_str(token.render().as_str());
                        for token in iter {
                            str.push_str(", ");
                            str.push_str(token.render().as_str());
                        }
                        str
                    }
                }
            }
            Error::AmbiguousUseOf { operator, .. } => {
                let mut str = String::from("ambiguous use of ");
                str.push_str(operator.render());
                str
            }
        }
    }

    pub fn report(&self, diagnostic: &mut Diagnostic) {
        diagnostic.item(
            Some(Location {
                source: self.source(),
                offset: Some(self.position()),
            }),
            Message {
                content: self.message(),
                addendum: None,
            },
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorName {
    Unexpected,
    AmbiguousUseOf(Spanned<Binop>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Parsed<A> {
    pub consumed: bool,
    pub result: Result<A, ErrorName>,
}

impl<A> Parsed<A> {
    pub fn and_then<B, F>(self, f: F) -> Parsed<B>
    where
        F: FnOnce(A) -> Parsed<B>,
    {
        match self.result {
            Err(err) => Parsed {
                consumed: self.consumed,
                result: Err(err),
            },
            Ok(a) => {
                let mut next = f(a);
                next.consumed = next.consumed || self.consumed;
                next
            }
        }
    }

    pub fn map<B, F>(self, f: F) -> Parsed<B>
    where
        F: FnOnce(A) -> B,
    {
        Parsed {
            consumed: self.consumed,
            result: self.result.map(f),
        }
    }

    fn pure(x: A) -> Self {
        Parsed {
            consumed: false,
            result: Ok(x),
        }
    }

    fn ambiguous_use_of(operator: Spanned<Binop>) -> Self {
        Parsed {
            consumed: false,
            result: Err(ErrorName::AmbiguousUseOf(operator)),
        }
    }

    fn unexpected(consumed: bool) -> Self {
        Parsed {
            consumed,
            result: Err(ErrorName::Unexpected),
        }
    }
}

#[macro_export]
macro_rules! apply {
    ($a:expr, $b:expr) => {
        match $a {
            None => None,
            Some(f) => match $b {
                None => None
                Some(x) => Some(f(x)),
            },
        }
    };
}

#[macro_export]
macro_rules! map0 {
    ($a:expr, $b:expr) => {{
        use $crate::Parsed;

        let b = $b;
        Parsed {
            consumed: b.consumed,
            result: b.result.map(|_| $a),
        }
    }};
}

#[macro_export]
macro_rules! map2 {
    ($f:expr, $a:expr, $b:expr) => {{
        use $crate::Parsed;

        let a = $a;
        match a.result {
            Err(err) => Parsed {
                consumed: a.consumed,
                result: Err(err),
            },
            Ok(val1) => {
                let b = $b;
                Parsed {
                    consumed: a.consumed || b.consumed,
                    result: b.result.map(|val2| $f(val1, val2)),
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! keep_right {
    ($a:expr, $b:expr) => {{
        use $crate::map2;
        map2!(|_, a| a, $a, $b)
    }};
}

#[macro_export]
macro_rules! keep_left {
    ($a:expr, $b:expr) => {{
        use $crate::map2;
        map2!(|a, _| a, $a, $b)
    }};
}

#[macro_export]
macro_rules! between {
    ($l:expr, $r:expr, $x:expr) => {{
        use $crate::{keep_left, keep_right};
        keep_right!($l, keep_left!($x, $r))
    }};
}

#[macro_export]
macro_rules! parse_string {
    ($p:ident, $s:expr) => {{
        use ipso_diagnostic::Source;
        use ipso_lex::{token::Token, Lexer};
        use ipso_parse::{keep_left, map2, Parser};

        let mut parser: Parser = Parser::new(
            Source::Interactive {
                label: String::from("(string)"),
            },
            Lexer::new(&$s),
        );
        let result = keep_left!(parser.$p(), parser.eof());
        parser.into_parse_error(result.result)
    }};
}

#[macro_export]
macro_rules! parse_str {
    ($p:ident, $s:expr) => {{
        use ipso_parse::parse_string;
        let s = String::from($s);
        parse_string!($p, s)
    }};
}

pub fn parse_string_at(source: Source, input: String) -> Result<Module, Error> {
    let mut parser: Parser = Parser::new(source, Lexer::new(&input));
    let result = keep_left!(module(&mut parser), parser.eof());
    parser.into_parse_error(result.result)
}

pub fn parse_string(input: String) -> Result<Module, Error> {
    let source = Source::Interactive {
        label: String::from("(string)"),
    };
    parse_string_at(source, input)
}

pub fn parse_file(filename: &Path) -> Result<Module, Error> {
    let input: String = {
        let mut content = String::new();
        let mut file: File = File::open(filename).unwrap();
        file.read_to_string(&mut content).unwrap();
        content
    };
    let source = Source::File {
        path: PathBuf::from(filename),
    };
    parse_string_at(source, input)
}

struct Expecting {
    bitset: FixedBitSet,
    indents: FnvHashSet<(Relation, usize)>,
}

impl Expecting {
    fn new() -> Self {
        Expecting {
            bitset: FixedBitSet::with_capacity(token::Name::num_variants()),
            indents: FnvHashSet::with_hasher(Default::default()),
        }
    }

    fn clear(&mut self) {
        self.bitset.clear();
        self.clear_indents();
    }

    fn clear_indents(&mut self) {
        self.indents.clear();
    }

    fn insert(&mut self, t: token::Name) {
        self.bitset.insert(t.to_int());
        if let token::Name::Indent(relation, n) = t {
            self.indents.insert((relation, n));
        }
    }

    fn into_btreeset(self) -> BTreeSet<token::Name> {
        let mut set = BTreeSet::new();
        let mut has_indents = false;
        for ix in self.bitset.ones() {
            if ix == token::INDENT_TAG
            // Indent(_)
            {
                has_indents = true;
            } else {
                set.insert(token::Name::from_int(ix).unwrap());
            }
        }
        if has_indents {
            set.extend(
                self.indents
                    .into_iter()
                    .map(|(relation, amount)| token::Name::Indent(relation, amount)),
            );
        }
        set
    }
}

pub struct Parser<'input> {
    source: Source,
    pos: usize,
    column: usize,
    indentation: Vec<usize>,
    expecting: Expecting,
    current: Option<Token>,
    input: Lexer<'input>,
}

#[macro_export]
macro_rules! many_ {
    ($x:expr) => {{
        use $crate::ErrorName;
        let mut error: Option<ErrorName> = None;
        let mut consumed = false;
        loop {
            let next = $x;
            match next.result {
                Err(err) => {
                    if next.consumed {
                        error = Some(err);
                    };
                    break;
                }
                Ok(_) => {
                    consumed = consumed || next.consumed;
                }
            }
        }

        Parsed {
            consumed,
            result: match error {
                None => Ok(()),
                Some(err) => Err(err),
            },
        }
    }};
}

#[macro_export]
macro_rules! many_with {
    ($vec:expr, $x:expr) => {{
        use $crate::{ErrorName, Parsed};

        let mut error: Option<ErrorName> = None;
        let mut acc: Vec<_> = $vec;
        let mut consumed = false;
        loop {
            let next = $x;
            match next.result {
                Err(err) => {
                    if next.consumed {
                        error = Some(err);
                    };
                    break;
                }
                Ok(val) => {
                    consumed = consumed || next.consumed;
                    acc.push(val)
                }
            }
        }

        Parsed {
            consumed,
            result: match error {
                None => Ok(acc),
                Some(err) => Err(err),
            },
        }
    }};
}

#[macro_export]
macro_rules! many {
    ($x:expr) => {{
        use $crate::many_with;
        many_with!(Vec::new(), $x)
    }};
}

#[macro_export]
macro_rules! choices {
    ($x:expr, $y:expr) => {{
        use $crate::Parsed;

        let first = $x;
        match first.result {
            Err(err) => {
                if first.consumed {
                    Parsed{ consumed: true, result: Err(err) }
                } else {
                    $y
                }
            }
            Ok(val) => Parsed{consumed: first.consumed, result: Ok(val)},
        }
    }};
    ($x:expr, $y:expr $(, $ys:expr)*) => {{
        use $crate::Parsed;

        let first = $x;
        match first.result {
            Err(err) => {
                if first.consumed {
                    Parsed{ consumed: true, result: Err(err) }
                } else {
                    choices!($y $(, $ys)*)
                }
            }
            Ok(val) => Parsed{consumed: first.consumed, result: Ok(val)},
        }
    }};
}

#[macro_export]
macro_rules! sep_by {
    ($x:expr, $sep:expr) => {{
        use $crate::many_with;
        choices!(
            $x.and_then(|first| { many_with!(vec![first], keep_right!($sep, $x)) }),
            Parsed::pure(Vec::new())
        )
    }};
}

#[macro_export]
macro_rules! optional {
    ($a:expr) => {{
        use $crate::Parsed;

        let first = $a;
        match first.result {
            Err(err) => {
                if first.consumed {
                    Parsed {
                        consumed: true,
                        result: Err(err),
                    }
                } else {
                    Parsed::pure(None)
                }
            }
            Ok(val) => Parsed {
                consumed: first.consumed,
                result: Ok(Some(val)),
            },
        }
    }};
}

#[macro_export]
macro_rules! spanned {
    ($self:expr, $x:expr) => {{
        use ipso_syntax as syntax;
        let pos = $self.pos;
        $x.map(|item| syntax::Spanned { pos, item })
    }};
}

impl<'input> Parser<'input> {
    pub fn new(source: Source, mut input: Lexer<'input>) -> Self {
        let current = input.next();
        Parser {
            source,
            pos: 0,
            column: 0,
            indentation: vec![],
            expecting: Expecting::new(),
            current,
            input,
        }
    }

    pub fn into_parse_error<A>(self, result: Result<A, ErrorName>) -> Result<A, Error> {
        match result {
            Ok(a) => Ok(a),
            Err(err) => Err(match err {
                ErrorName::Unexpected => Error::Unexpected {
                    source: self.source,
                    pos: self.pos,
                    expecting: self.expecting.into_btreeset(),
                },
                ErrorName::AmbiguousUseOf(operator) => Error::AmbiguousUseOf {
                    source: self.source,
                    pos: operator.pos,
                    operator: operator.item,
                },
            }),
        }
    }

    pub fn eof(&mut self) -> Parsed<()> {
        self.token(&token::Data::Eof)
    }

    fn consume(&mut self) -> Parsed<()> {
        match &self.current {
            None => Parsed::unexpected(false),
            Some(_) => {
                self.current = self.input.next();
                match &self.current {
                    None => {}
                    Some(token) => {
                        self.pos = token.pos;
                        self.column = token.column;
                    }
                };
                self.expecting.clear();
                Parsed {
                    consumed: true,
                    result: Ok(()),
                }
            }
        }
    }

    fn comment(&mut self) -> Parsed<()> {
        self.expecting.insert(token::Name::Comment);
        match &self.current {
            None => Parsed::unexpected(false),
            Some(token) => match token.data {
                token::Data::Comment { .. } => map0!((), self.consume()),
                _ => Parsed::unexpected(false),
            },
        }
    }

    fn keyword(&mut self, expected: &Keyword) -> Parsed<()> {
        self.expecting.insert(token::Name::Keyword(*expected));
        match &self.current {
            None => Parsed::unexpected(false),
            Some(actual) => match &actual.data {
                token::Data::Ident(id) => {
                    if expected.matches(id) {
                        map0!((), self.consume())
                    } else {
                        Parsed::unexpected(false)
                    }
                }
                _ => Parsed::unexpected(false),
            },
        }
    }

    fn token(&mut self, expected: &token::Data) -> Parsed<()> {
        self.expecting.insert(expected.name());
        match &self.current {
            Some(actual) if actual.data == *expected => self
                .consume()
                .and_then(|_| map0!((), many_!(self.comment()))),
            _ => Parsed::unexpected(false),
        }
    }

    fn ident(&mut self) -> Parsed<Rc<str>> {
        self.expecting.insert(token::Name::Ident);
        match &self.current {
            Some(token) => match &token.data {
                token::Data::Ident(s) if !syntax::is_keyword(s) => match s.chars().next() {
                    Some(c) if c.is_lowercase() => {
                        let s = s.clone();
                        self.consume()
                            .and_then(|_| map0!(s, many_!(self.comment())))
                    }
                    _ => Parsed::unexpected(false),
                },
                _ => Parsed::unexpected(false),
            },
            None => Parsed::unexpected(false),
        }
    }

    fn ident_owned(&mut self) -> Parsed<String> {
        self.ident().map(|i| String::from(i.as_ref()))
    }

    fn ctor(&mut self) -> Parsed<Rc<str>> {
        self.expecting.insert(token::Name::Ctor);
        match &self.current {
            Some(token) => match &token.data {
                token::Data::Ident(s) if !syntax::is_keyword(s) => match s.chars().next() {
                    Some(c) if c.is_uppercase() => {
                        let s = s.clone();
                        self.consume()
                            .and_then(|_| map0!(s, many_!(self.comment())))
                    }
                    _ => Parsed::unexpected(false),
                },
                _ => Parsed::unexpected(false),
            },
            None => Parsed::unexpected(false),
        }
    }

    fn ctor_owned(&mut self) -> Parsed<String> {
        self.ctor().map(|s| String::from(s.as_ref()))
    }

    fn int(&mut self) -> Parsed<i32> {
        self.expecting.insert(token::Name::Int);
        (match &self.current {
            Some(token) => match &token.data {
                token::Data::Int {
                    sign,
                    value,
                    length: _,
                } => Parsed::pure((*sign, *value)),
                _ => Parsed::unexpected(false),
            },
            None => Parsed::unexpected(false),
        })
        .and_then(|(sign, value)| {
            map0!(
                match sign {
                    Sign::Negative => -(value as i32),
                    Sign::None => value as i32,
                },
                self.consume()
            )
        })
    }

    /// ```
    /// use std::rc::Rc;
    /// use ipso_diagnostic::Source;
    /// use ipso_lex::{self, token};
    /// use ipso_parse::{Error, parse_str};
    ///
    /// assert_eq!(parse_str!(char, "\'a\'"), Ok('a'));
    ///
    /// assert_eq!(parse_str!(char, "\'\\\\\'"), Ok('\\'));
    ///
    /// assert_eq!(parse_str!(char, "\'\\n\'"), Ok('\n'));
    ///
    /// assert_eq!(parse_str!(char, "'"), Err(Error::Unexpected {
    ///     source: Source::Interactive{label: String::from("(string)")},
    ///     pos: 1,
    ///     expecting: vec![token::Name::Char, token::Name::Comment].into_iter().collect(),
    /// }));
    ///
    /// assert_eq!(parse_str!(char, "\'\\\'"), Err(Error::Unexpected {
    ///     source: Source::Interactive{label: String::from("(string)")},
    ///     pos: 3,
    ///     expecting: vec![token::Name::SingleQuote].into_iter().collect(),
    /// }));
    ///
    /// assert_eq!(parse_str!(char, "\'\\"), Err(Error::Unexpected {
    ///     source: Source::Interactive{label: String::from("(string)")},
    ///     pos: 1,
    ///     expecting: vec![token::Name::Char, token::Name::Comment].into_iter().collect(),
    /// }));
    ///
    /// assert_eq!(parse_str!(char, "\'\\~\'"), Err(Error::Unexpected {
    ///     source: Source::Interactive{label: String::from("(string)")},
    ///     pos: 2,
    ///     expecting: vec![token::Name::Char, token::Name::Comment].into_iter().collect(),
    /// }));
    /// ```
    pub fn char(&mut self) -> Parsed<char> {
        between!(
            self.token(&token::Data::SingleQuote),
            self.token(&token::Data::SingleQuote),
            {
                self.expecting.insert(token::Name::Char);
                match &self.current {
                    Some(token) => match token.data {
                        token::Data::Char { value, length: _ } => {
                            map0!(value, self.consume())
                        }
                        _ => Parsed::unexpected(false),
                    },
                    None => Parsed::unexpected(false),
                }
            }
        )
    }

    pub fn string(&mut self) -> Parsed<String> {
        self.expecting.insert(token::Name::String);

        let str = match &self.current {
            Some(current) => match &current.data {
                token::Data::String { value, .. } => value.clone(),
                _ => return Parsed::unexpected(false),
            },
            None => return Parsed::unexpected(false),
        };
        self.consume();

        Parsed::pure(str)
    }
}
