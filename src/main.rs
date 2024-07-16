mod tokens;
mod lexer;
mod parser;

fn main() {
    parser::Parser::new("test.txt").parse().unwrap();
}
