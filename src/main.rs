mod parser;
use chumsky::Parser;
use parser::parser;

fn main() {
    let src = "1 + hello + world";
    match parser().parse(src) {
        Ok(ast) => {
            dbg!(&ast);
        }
        Err(errs) => errs.into_iter().for_each(|e| println!("{:?}", e)),
    };
}
