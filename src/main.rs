use std::collections::HashMap;

mod lexer;
mod parser;
mod tokens;

#[derive(Debug)]
pub struct Process {
    id: String,
    input: Vec<(String, u64)>,
    output: Vec<(String, u64)>,
    time: u64,
}

#[derive(Debug)]
pub struct Data {
    pub stocks: HashMap<String, u64>,
    pub processes: Vec<Process>,
    pub objectives: Vec<String>,
}

fn main() {
    let mut parser = parser::Parser::new("test2.txt");
    match parser.parse() {
        Err(err) => {
            println!("\n\nERROR !!!!!: {:?}", err);
            return;
        }
        _ => {}
    };
    let x = Data {
        stocks: parser.stocks,
        processes: parser.process,
        objectives: parser.optimize.unwrap(),
    };
    println!("{:?}", x);
}
