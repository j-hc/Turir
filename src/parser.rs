use crate::lexer::{Lexer, Token, TokenKind, TokenResult};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum Dir {
    Left,
    Right,
}

impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dir = match self {
            Dir::Left => "<-",
            Dir::Right => "->",
        };
        write!(f, "{dir}")
    }
}

#[derive(Debug)]
pub struct RunCmd<'c> {
    pub tape: Vec<&'c str>,
    pub state: &'c str,
}

impl<'c> fmt::Display for RunCmd<'c> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#run [ ")?;
        let (last, t) = self.tape.split_last().expect("tape cannot be empty");
        for t in t {
            write!(f, "{t} ")?;
        }
        write!(f, "{last} ] {state}", state = self.state)?;
        Ok(())
    }
}
#[derive(Debug)]
pub struct HaltCmd<'c> {
    pub states: Vec<&'c str>,
}

#[derive(Debug)]
pub struct ParseErr<'c, 'k> {
    pub expected: &'k [TokenKind],
    pub got: TokenResult<'c>,
}
impl<'c, 'k> Error for ParseErr<'c, 'k> {}

impl<'c, 'k> fmt::Display for ParseErr<'c, 'k> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let loc = match &self.got {
            TokenResult::Eof { loc } => loc,
            TokenResult::Valid(Token { loc, .. }) => loc,
            TokenResult::Unknown { loc, .. } => loc,
            TokenResult::UnclosedStr { loc } => loc,
        };
        write!(f, "{loc}: Expected ")?;

        let (last, ks) = self.expected.split_last().unwrap();
        for k in ks.iter() {
            write!(f, "{} or ", k.to_str())?;
        }
        write!(f, "{} but got ", last.to_str())?;
        match &self.got {
            TokenResult::Eof { .. } => write!(f, "EOF"),
            TokenResult::Valid(Token { text, kind, .. }) => {
                write!(f, "{} {:?}", kind.to_str(), text)
            }
            TokenResult::Unknown { text, .. } => write!(f, "unknown token `{}`", text),
            TokenResult::UnclosedStr { .. } => write!(f, "unclosed string"),
        }
    }
}

pub struct Parser<'c> {
    lexer: Lexer<'c>,
}
impl<'c> Parser<'c> {
    pub fn new(lexer: Lexer<'c>) -> Self {
        Self { lexer }
    }

    pub fn peek_token(&mut self) -> TokenResult<'c> {
        self.lexer.peek_token()
    }

    pub fn skip_token(&mut self) {
        let _ = self.lexer.next_token();
    }

    fn expect_token<'k>(&mut self, kinds: &'k [TokenKind]) -> Result<Token<'c>, ParseErr<'c, 'k>> {
        match self.lexer.next_token() {
            TokenResult::Valid(tok @ Token { kind, .. }) if kinds.iter().any(|&k| kind == k) => {
                Ok(tok)
            }
            tr => Err(ParseErr {
                expected: kinds,
                got: tr,
            })?,
        }
    }

    pub fn parse_instr<'k>(&mut self) -> Result<Instr<&'c str, &'c str>, ParseErr<'c, 'k>> {
        use TokenKind::*;
        let state = self.expect_token(&[Symbol])?.text;
        let read = self.expect_token(&[Symbol])?.text;
        let write = self.expect_token(&[Symbol])?.text;

        let dir = match self.expect_token(&[LeftArrow, RightArrow])?.kind {
            LeftArrow => Dir::Left,
            RightArrow => Dir::Right,
            _ => unreachable!(),
        };

        let next_state = self.expect_token(&[Symbol])?.text;
        let _ = self.expect_token(&[NewLine])?;
        Ok(Instr {
            state,
            read,
            write,
            dir,
            next_state,
        })
    }

    pub fn parse_cmd_run<'k>(&mut self) -> Result<RunCmd<'c>, ParseErr<'c, 'k>> {
        use TokenKind::*;

        let _ = self.expect_token(&[Cmd])?;
        let _ = self.expect_token(&[Bra])?;

        let mut tape = Vec::new();
        loop {
            let token = self.expect_token(&[Symbol, Ket])?;
            match token.kind {
                Symbol => tape.push(token.text),
                Ket => break,
                _ => unreachable!(),
            }
        }
        let state = self.expect_token(&[Symbol])?.text;
        let _ = self.expect_token(&[NewLine])?;
        Ok(RunCmd { tape, state })
    }

    pub fn parse_cmd_halt<'k>(&mut self) -> Result<HaltCmd<'c>, ParseErr<'c, 'k>> {
        use TokenKind::*;

        let _ = self.expect_token(&[Cmd])?;

        let mut states = Vec::new();
        loop {
            let token = self.expect_token(&[NewLine, Symbol])?;
            match token.kind {
                Symbol => states.push(token.text),
                NewLine => break,
                _ => unreachable!(),
            }
        }

        Ok(HaltCmd { states })
    }
}

pub struct Program<'c> {
    pub runs: Vec<RunCmd<'c>>,
    pub halt_syms: Vec<&'c str>,
    pub program: Vec<Instr<&'c str, &'c str>>,
}

#[derive(Debug)]
pub struct Instr<St, Sym> {
    pub state: St,
    pub read: Sym,
    pub write: Sym,
    pub dir: Dir,
    pub next_state: St,
}

impl<St: fmt::Display, Sym: fmt::Display> fmt::Display for Instr<St, Sym> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.state, self.read, self.write, self.dir, self.next_state
        )
    }
}

pub fn parse_source<'c, 'k>(
    content: &'c [u8],
    file: &'static str,
) -> Result<Program<'c>, ParseErr<'c, 'k>> {
    let lexer = Lexer::new(content, file);
    let mut parser = Parser::new(lexer);

    use TokenKind::*;

    let mut program: Vec<Instr<&str, &str>> = Vec::new();
    let mut runs: Vec<RunCmd> = Vec::new();
    let mut halt_syms: Vec<&str> = Vec::new();
    loop {
        let token = match parser.peek_token() {
            TokenResult::Eof { .. } => break,
            TokenResult::Valid(t) => t,
            got => {
                return Err(ParseErr {
                    expected: &[Symbol, Cmd],
                    got,
                })
            }
        };

        match token {
            Token {
                kind: Cmd,
                text: "#run",
                ..
            } => runs.push(parser.parse_cmd_run()?),
            Token {
                kind: Cmd,
                text: "#halt",
                ..
            } => halt_syms = parser.parse_cmd_halt()?.states,
            Token { kind: Symbol, .. } => program.push(parser.parse_instr()?),
            Token { kind: NewLine, .. } => parser.skip_token(),
            tok => unreachable!("{tok:?}"),
        }
    }
    if halt_syms.is_empty() {
        halt_syms.push("HALT");
    }

    Ok(Program {
        runs,
        halt_syms,
        program,
    })
}
