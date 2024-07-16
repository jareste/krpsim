use crate::tokens::Token;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

const DELIMITER: &str = " \t\n\r();:";

struct Lexer {
    source: String,
    current: usize,
}

pub fn lex(filename: &'static str) -> Result<Vec<Token>, io::Error> {
    let mut tokens: Vec<Token> = Vec::new();
    let lines = read_lines(filename)?;
    for line in lines.flatten() {
        if line.starts_with("#") {
            continue;
        }
        Lexer::new(line).tokenize(&mut tokens);
    }
    return Ok(tokens);
}

impl Lexer {
    fn new(source: String) -> Self {
        return Self { source, current: 0 };
    }

    fn tokenize(&mut self, tokens: &mut Vec<Token>) {
        match self.source.chars().nth(self.current) {
            None => tokens.push(Token::NewLine),
            Some(c) => match c {
                ':' => self.advance(Token::Colon, tokens),
                ';' => self.advance(Token::Semicolon, tokens),
                '(' => self.advance(Token::LeftParen, tokens),
                ')' => self.advance(Token::RightParen, tokens),
                ' ' => self.tokenize(tokens),
                _ => self.identifier(tokens),
            },
        }
    }

    fn advance(&mut self, token: Token, tokens: &mut Vec<Token>) {
        match &token {
            Token::Identifier(i) => self.current += i.len(),
            Token::Invalid(i) => self.current += i.len(),
            Token::Number(n) => self.current += (n.checked_ilog10().unwrap_or(0) + 1) as usize,
            Token::Time => self.current += 4,
            Token::Optimize => self.current += 8,
            _ => self.current += 1,
        }
        tokens.push(token);
        self.tokenize(tokens);
    }

    fn identifier(&mut self, tokens: &mut Vec<Token>) {
        let ident: String = self
            .source
            .chars()
            .skip(self.current)
            .take_while(|&c| !DELIMITER.contains(c))
            .collect();
        match ident.as_str() {
            "optimize" => self.advance(Token::Optimize, tokens),
            "time" => self.advance(Token::Time, tokens),
            "\n" => self.advance(Token::NewLine, tokens),
            x if x.starts_with(|c: char| c.is_digit(10)) => match ident.parse::<u64>() {
                Ok(n) => self.advance(Token::Number(n), tokens),
                _ => self.advance(Token::Invalid(ident), tokens),
            },
            x if x.starts_with(|c: char| !c.is_alphabetic()) => {
                self.advance(Token::Invalid(ident), tokens)
            }
            _ => self.advance(Token::Identifier(ident), tokens),
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
