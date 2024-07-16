use crate::lexer;
use std::collections::HashMap;

use crate::tokens::Token;

struct Process {
    id: String,
    input: Vec<(String, u64)>,
    output: Vec<(String, u64)>,
    time: u64,
}

pub struct Parser {
    current: usize,
    tokens: Vec<Token>,

    stocks: HashMap<String, u64>,
    process: Vec<Process>,
    optimize: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum Error {
    ExpectedLine,
    MissingOptimize,
    MissingStocks,
    MissingProcess,
    DuplicatedOptimize,
    DuplicatedIdentifier,
    UnexpectedEOF,
    UnexpectedToken,
}

impl Parser {
    pub fn new(file: &'static str) -> Self {
        let tokens = lexer::lex(file).unwrap();
        println!("{:?}", tokens);
        Self {
            current: 0,
            tokens,

            stocks: HashMap::new(),
            process: Vec::new(),
            optimize: None,
        }
    }

    pub fn parse(&mut self) -> Result<(), Error> {
        if self.current >= self.tokens.len() {
            if self.optimize.is_none() {
                return Err(Error::MissingOptimize);
            }
            if self.stocks.is_empty() {
                return Err(Error::MissingStocks);
            }
            if self.process.is_empty() {
                return Err(Error::MissingProcess);
            }
            return Ok(());
        }
        self.parse_line()?;
        self.parse()
    }

    fn parse_line(&mut self) -> Result<(), Error> {
        match self.peek() {
            None => Err(Error::ExpectedLine),
            Some(t) => match t {
                Token::Optimize => Ok(self.parse_optimize()?),
                Token::Identifier(_) => {
                    let ident = self.consume_ident()?.to_string();
                    self.consume(Token::Colon)?;
                    match self.peek() {
                        None => Err(Error::UnexpectedEOF),
                        Some(token) => match token {
                            Token::LeftParen => Ok(self.parse_process(ident)?),
                            Token::Number(_) => Ok(self.parse_stock(ident)?),
                            _ => Err(Error::UnexpectedToken),
                        },
                    }
                }
                _ => Err(Error::UnexpectedToken),
            },
        }
    }

    fn parse_process(&mut self, id: String) -> Result<(), Error> {
        // TODO: Check that the ident is not in stocks
        // TODO: Check that the input is not in stocks
        let input = self.parse_tuple()?;
        self.consume(Token::Colon)?;
        // TODO: Create outputs if is not in stocks
        let output = self.parse_tuple()?;
        self.consume(Token::Colon)?;
        let time = self.consume_number()?;
        Ok(self.process.push(Process {
            id,
            input,
            output,
            time,
        }))
    }

    fn parse_tuple(&mut self) -> Result<Vec<(String, u64)>, Error> {
        let mut res: Vec<(String, u64)> = Vec::new();

        self.consume(Token::LeftParen)?;
        loop {
            let ident = self.consume_ident()?.to_string();
            self.consume(Token::Colon)?;
            let n = self.consume_number()?;
            res.push((ident, n));
            match self.peek() {
                None => return Err(Error::UnexpectedEOF),
                Some(token) => match token {
                    Token::RightParen => return Ok(res),
                    Token::Semicolon => {}
                    _ => return Err(Error::UnexpectedToken),
                },
            }
        }
    }

    fn parse_stock(&mut self, ident: String) -> Result<(), Error> {
        let n = self.consume_number()?;
        match self.stocks.insert(ident.to_string(), n) {
            Some(_) => Err(Error::DuplicatedIdentifier),
            _ => Ok(self.consume(Token::NewLine)?),
        }
    }

    fn parse_optimize(&mut self) -> Result<(), Error> {
        let mut res: Vec<String> = Vec::new();

        if self.optimize.is_some() {
            return Err(Error::DuplicatedOptimize);
        }
        self.advance();
        self.consume(Token::Colon)?;
        self.consume(Token::LeftParen)?;
        loop {
            match self.peek() {
                None => return Err(Error::UnexpectedEOF),
                Some(t) => match t {
                    Token::Time => {
                        self.advance();
                        res.push("time".to_string());
                    }
                    Token::Identifier(_) => {
                        res.push(self.consume_ident()?.to_string());
                    }
                    _ => return Err(Error::UnexpectedToken),
                },
            }
            match self.peek() {
                None => {
                    return Err(Error::UnexpectedEOF);
                }
                Some(token) => match token {
                    Token::RightParen => {
                        self.advance();
                        break;
                    }
                    Token::Semicolon => {
                        self.advance();
                    }
                    _ => return Err(Error::UnexpectedToken),
                },
            }
        }
        self.consume(Token::NewLine)?;
        self.optimize = Some(res);
        Ok(())
    }

    fn consume(&mut self, token: Token) -> Result<(), Error> {
        match self.peek() {
            Some(t) if *t == token => {
                self.advance();
                Ok(())
            }
            _ => Err(Error::UnexpectedToken),
        }
    }

    fn consume_ident(&mut self) -> Result<&String, Error> {
        match self.peek() {
            Some(t) => match t {
                Token::Identifier(ident) => Ok(ident),
                _ => Err(Error::MissingProcess),
            },
            _ => Err(Error::MissingProcess),
        }
    }

    fn consume_number(&mut self) -> Result<u64, Error> {
        match self.peek() {
            Some(t) => match t {
                Token::Number(n) => Ok(*n),
                _ => Err(Error::MissingProcess),
            },
            _ => Err(Error::MissingProcess),
        }
    }

    fn advance(&mut self) -> &Token {
        let token = self
            .tokens
            .get(self.current)
            .expect("advance: Expected Token");
        self.current += 1;
        token
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }
}
