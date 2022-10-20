
extern crate test;

use std::str::Chars;
use self::TokenKind::*;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub len: usize,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenKind<'a> {
    Identifier(&'a str),
    Numerical(&'a str),
    String(&'a str),

    // One-char tokens:
    /// ";"
    Semi,
    /// ","
    Comma,
    /// "."
    Dot,
    /// "("
    OpenParen,
    /// ")"
    CloseParen,
    /// "{"
    OpenBrace,
    /// "}"
    CloseBrace,
    /// "["
    OpenBracket,
    /// "]"
    CloseBracket,
    /// "@"
    At,
    /// "#"
    Pound,
    /// "~"
    Tilde,
    /// "?"
    Question,
    /// ":"
    Colon,
    /// "$"
    Dollar,
    /// "="
    Eq,
    /// "!"
    Bang,
    /// "<"
    Lt,
    /// ">"
    Gt,
    /// "-"
    Minus,
    /// "&"
    And,
    /// "|"
    Or,
    /// "+"
    Plus,
    /// "*"
    Star,
    /// "/"
    Slash,
    /// "^"
    Caret,
    /// "%"
    Percent,

    /// Unknown token, not expected by the lexer, e.g. "№"
    Unknown,
    EOF,
}

/// Peekable iterator over a char sequence.
///
/// Next characters can be peeked via `first` method,
/// and position can be shifted forward via `bump` method.
pub(crate) struct Tokenizer<'a> {
    pos: usize,
    input: &'a str,
    /// Iterator over chars. Slightly faster than a &str.
    chars: Chars<'a>,
}


impl<'a> Tokenizer<'_> {
    pub(crate) fn new(input: &str) -> Tokenizer {
        Tokenizer {
            pos: 0,
            input,
            chars: input.chars(),
        }
    }
}

impl<'a> Token<'_> {
    fn new(kind: TokenKind, len: usize) -> Token {
        Token { kind, len }
    }
}

impl <'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.offset();
        let first_char = self.bump()?;
        let token_kind = match first_char {
            // Identifiers
            c if c.is_alphabetic() || c == '_' => {
                // first char must be alphaetic, but consecutive chars can have integers
                self.skip_while(|c| c.is_alphanumeric() || c == '_');
                Identifier(self.read_str(start, self.offset()))
            },

            // Integers & Floats
            '0'..='9' => {
                self.skip_while(|c| c.is_digit(10) || c == '.');
                let strval = self.read_str(start, self.offset());
                Numerical(strval)
            }

            // String values
            '"' => {
                self.skip_while(|c| c != '"');
                self.bump();
                String(self.read_str(start+1, self.offset() - 1))
            }

            // Whitespace (skipped)
            c if is_whitespace(c) => return self.next(),

            // One-symbol tokens.
            ';' => Semi,
            ',' => Comma,
            '.' => Dot,
            '(' => OpenParen,
            ')' => CloseParen,
            '{' => OpenBrace,
            '}' => CloseBrace,
            '[' => OpenBracket,
            ']' => CloseBracket,
            '@' => At,
            '#' => Pound,
            '~' => Tilde,
            '?' => Question,
            ':' => Colon,
            '$' => Dollar,
            '=' => Eq,
            '!' => Bang,
            '<' => Lt,
            '>' => Gt,
            '-' => Minus,
            '&' => And,
            '|' => Or,
            '+' => Plus,
            '*' => Star,
            '^' => Caret,
            '%' => Percent,
            '/' => Slash,
            _ => TokenKind::Unknown,
        };

        Some(Token::new(token_kind, self.offset() - start))
    }
}


impl<'a> Tokenizer<'a> {
    /// Peeks the next symbol from the input stream without consuming it.
    /// If requested position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file,
    /// it should be checked with `is_eof` method.
    pub(crate) fn first(&self) -> Option<char> {
        // `.next()` optimizes better than `.nth(0)`
        self.chars.clone().next()
    }

    /// Checks if there is nothing more to consume.
    pub(crate) fn is_eof(&self) -> bool {
        self.offset() >= self.input.len()
    }

    /// Moves to the next character.
    pub(crate) fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Returns current offset into the source file
    pub(crate) fn offset(&self) -> usize {
        self.pos
    }

    /// Takes a string slice from and to given offsets, respectively
    pub(crate) fn read_str(&self, from: usize, to: usize) -> &'a str {
        &self.input[from..to]
    }

    /// Eats symbols while predicate returns true or until the end of file is reached.
    pub(crate) fn skip_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        // It was tried making optimized version of this for eg. line comments, but
        // LLVM can inline all of this and compile it down to fast iteration over bytes.
        while !self.is_eof() && predicate(self.first().unwrap()) {
            self.bump();
        }
    }

}

/// True if `c` is considered a whitespace according to Rust language definition.
/// See [Rust language reference](https://doc.rust-lang.org/reference/whitespace.html)
/// for definitions of these classes.
pub fn is_whitespace(c: char) -> bool {
    // This is Pattern_White_Space.
    //
    // Note that this set is stable (ie, it doesn't change with different
    // Unicode versions), so it's ok to just hard-code the values.

    matches!(
        c,
        // Usual ASCII suspects
        '\u{0009}'   // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_lexing() {
        let mut tokenizer = Tokenizer::new("let foobar = 5;");
        assert_eq!(tokenizer.next(), Some(Token{ kind: Identifier("let"), len: 3}));
        assert_eq!(tokenizer.next(), Some(Token{ kind: Identifier("foobar"), len: 6}));
        assert_eq!(tokenizer.next(), Some(Token{ kind: Eq, len: 1}));
        assert_eq!(tokenizer.next(), Some(Token{ kind: Numerical("5"), len: 1}));
        assert_eq!(tokenizer.next(), Some(Token{ kind: Semi, len: 1}));
        assert_eq!(tokenizer.next(), None);
        assert_eq!(tokenizer.next(), None);
    }

    #[bench]
    fn bench_tokenizer(b: &mut Bencher) {
        let input = std::fs::read_to_string("./examples/cargo-sample.rs").unwrap();
        b.iter(|| {
            let v = Tokenizer::new(&input).collect::<Vec<Token>>();
            
        });
    }
}