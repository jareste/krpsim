mod tokens;
mod lexer;

fn main() {
    let tokens = lexer::lex("test.txt").unwrap();
    for t in tokens {
        println!("{:?}", t)
    }
}
