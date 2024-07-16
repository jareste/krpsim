#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(u64),
    Optimize,
    Colon,
    Semicolon,
    LeftParen,
    RightParen,
    NewLine,
    Time,
    Invalid(String),
}
