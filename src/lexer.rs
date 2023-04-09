// CUR_STATE READ WRITE DIRECTION NEXT_STATE
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind {
    Symbol,
    LeftArrow,
    RightArrow,
    Cmd,
    Bra,
    Ket,
    NewLine,
}

impl TokenKind {
    pub const fn to_str(self) -> &'static str {
        match self {
            TokenKind::Symbol => "Symbol",
            TokenKind::LeftArrow => "<-",
            TokenKind::RightArrow => "->",
            TokenKind::Cmd => "Cmd",
            TokenKind::Bra => "[",
            TokenKind::Ket => "]",
            TokenKind::NewLine => "new line",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Loc {
    file: &'static str,
    row: usize,
    col: usize,
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.row + 1, self.col + 1)
    }
}

#[derive(Debug, Clone)]
pub struct Token<'c> {
    pub kind: TokenKind,
    pub text: &'c str,
    pub loc: Loc,
}

#[derive(Debug)]
pub struct Lexer<'c> {
    content: &'c [u8],
    file: &'static str,
    cur: usize,
    bol: usize,
    row: usize,
}

impl<'c> Lexer<'c> {
    pub fn new(content: &'c [u8], file: &'static str) -> Self {
        Self {
            content,
            file,
            cur: 0,
            bol: 0,
            row: 0,
        }
    }
}

#[derive(Debug)]
pub enum TokenResult<'c> {
    Eof { loc: Loc },
    Unknown { text: &'c str, loc: Loc },
    UnclosedStr { loc: Loc },
    Valid(Token<'c>),
}

const LITERALS: [(&str, TokenKind); 5] = [
    ("->", TokenKind::RightArrow),
    ("<-", TokenKind::LeftArrow),
    ("[", TokenKind::Bra),
    ("]", TokenKind::Ket),
    ("\n", TokenKind::NewLine),
];

impl<'c> Lexer<'c> {
    fn text_from_content(&self, start: usize) -> &'c str {
        unsafe { std::str::from_utf8_unchecked(&self.content[start..self.cur]) }
    }

    fn trim_left(&mut self) {
        while !self.exhausted() && matches!(self.content[self.cur], b'\t' | b'\x0C' | b'\r' | b' ')
        {
            self.skip_n(1);
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        self.content[self.cur..]
            .iter()
            .zip(s.as_bytes())
            .all(|c| c.0 == c.1)
    }

    fn skip_n(&mut self, n: usize) {
        assert!(!self.exhausted());
        for _ in 0..n {
            let c = self.content[self.cur];
            self.cur += 1;
            if c == b'\n' {
                self.bol = self.cur;
                self.row += 1;
            }
        }
    }

    pub fn loc(&self) -> Loc {
        Loc {
            file: self.file,
            row: self.row,
            col: self.cur - self.bol,
        }
    }

    pub fn peek_token(&mut self) -> TokenResult<'c> {
        let (cur, bol, row) = (self.cur, self.bol, self.row);
        let t = self.next_token();
        (self.cur, self.bol, self.row) = (cur, bol, row);
        t
    }

    fn skip_until(&mut self, c: u8) {
        while !self.exhausted() && self.content[self.cur] != c {
            self.skip_n(1);
        }
    }

    fn extract_token<P>(
        &mut self,
        pred: P,
        kind: TokenKind,
        start: usize,
        loc: Loc,
    ) -> TokenResult<'c>
    where
        P: Fn(u8) -> bool,
    {
        self.skip_n(1);
        while !self.exhausted() && pred(self.content[self.cur]) {
            self.skip_n(1);
        }
        TokenResult::Valid(Token {
            kind,
            loc,
            text: self.text_from_content(start),
        })
    }

    fn extract_string_token(&mut self, start: usize, loc: Loc) -> TokenResult<'c> {
        loop {
            self.skip_n(1);
            if self.content[self.cur] == b'\n' || self.exhausted() {
                return TokenResult::UnclosedStr { loc };
            }
            if self.content[self.cur] == b'\'' {
                break;
            }
        }
        let tok = TokenResult::Valid(Token {
            kind: TokenKind::Symbol,
            text: self.text_from_content(start + 1),
            loc,
        });
        self.skip_n(1);
        tok
    }

    pub fn next_token(&mut self) -> TokenResult<'c> {
        self.trim_left();

        let start = self.cur;
        let loc = self.loc();

        if self.exhausted() {
            return TokenResult::Eof { loc };
        }
        if self.starts_with("//") {
            self.skip_until(b'\n');
        }

        if self.content[self.cur] == b'\'' {
            return self.extract_string_token(start, loc);
        }

        if self.content[self.cur] == b'#' {
            return self.extract_token(is_symbol, TokenKind::Cmd, start, loc);
        }

        if is_symbol(self.content[self.cur]) {
            return self.extract_token(is_symbol, TokenKind::Symbol, start, loc);
        }

        for (lit, kind) in LITERALS {
            if self.starts_with(lit) {
                self.skip_n(lit.len());
                return TokenResult::Valid(Token {
                    kind,
                    loc,
                    text: self.text_from_content(start),
                });
            }
        }

        self.skip_n(1);
        TokenResult::Unknown {
            text: self.text_from_content(start),
            loc,
        }
    }

    fn exhausted(&self) -> bool {
        self.content.len() <= self.cur
    }
}

fn is_symbol(s: u8) -> bool {
    let lits = LITERALS
        .iter()
        .all(|(lit, _)| lit.as_bytes().iter().all(|&c| c != s));
    lits && !s.is_ascii_whitespace() && s != b'\''
}
