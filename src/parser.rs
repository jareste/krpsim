use crate::lexer;
use crate::Process;
use std::collections::HashMap;

use crate::tokens::Token;

#[derive(Debug)]
pub struct Parser {
    current: usize,
    tokens: Vec<Token>,

    pub stocks: HashMap<String, u64>,
    pub process: Vec<Process>,
    pub optimize: Option<Vec<String>>,
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
    UnexpectedToken(Token),
    UndefiendStock(String),
    ExpectedToken(Token, Token),
}

impl Parser {
    pub fn new(file: &'static str) -> Self {
        let tokens = lexer::lex(file).unwrap();
        // println!("{:?}", tokens);
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
                            _ => Err(Error::UnexpectedToken(token.clone())),
                        },
                    }
                }
                Token::NewLine => Ok(self.consume(Token::NewLine)?),
                _ => Err(Error::UnexpectedToken(t.clone())),
            },
        }
    }

    fn parse_process(&mut self, id: String) -> Result<(), Error> {
        if self.stocks.get(&id).is_some() {
            return Err(Error::DuplicatedIdentifier);
        }
        if self.process.iter().any(|x| x.id == id) {
            return Err(Error::DuplicatedIdentifier);
        }
        let input = self.parse_tuple()?;
        // NOTE: Probably there is a better way
        // for (k, _) in input.iter() {
        //     if !self.stocks.contains_key(k) {
        //         return Err(Error::UndefiendStock(k.to_string()));
        //     }
        // }
        self.consume(Token::Colon)?;
        let output = self.parse_tuple()?;
        for (k, _) in output.iter() {
            if !self.stocks.contains_key(k) {
                self.stocks.insert(k.to_string(), 0);
            }
        }
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
                    Token::RightParen => {
                        self.advance();
                        return Ok(res);
                    }
                    Token::Semicolon => { self.advance(); }
                    _ => return Err(Error::UnexpectedToken(token.clone())),
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
                    _ => return Err(Error::UnexpectedToken(t.clone())),
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
                    _ => return Err(Error::UnexpectedToken(token.clone())),
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
            Some(t) => Err(Error::ExpectedToken(token.clone(), t.clone())),
            _ => Err(Error::UnexpectedEOF),
        }
    }

    fn consume_ident(&mut self) -> Result<&String, Error> {
        match self.tokens.get(self.current) {
            Some(t) => match t {
                Token::Identifier(ident) => {
                    self.current += 1;
                    Ok(ident)
                }
                _ => Err(Error::ExpectedToken(
                    Token::Identifier("".to_string()),
                    t.clone(),
                )),
            },
            _ => Err(Error::UnexpectedEOF),
        }
    }

    fn consume_number(&mut self) -> Result<u64, Error> {
        match self.tokens.get(self.current) {
            Some(t) => match t {
                Token::Number(n) => {
                    self.current += 1;
                    Ok(*n)
                }
                _ => Err(Error::ExpectedToken(Token::Number(0), t.clone())),
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
