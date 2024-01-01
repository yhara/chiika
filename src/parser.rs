use chumsky::prelude::*;

#[derive(PartialEq, Debug)]
pub enum BinOp {
    Add,
    Sub,
}

#[derive(PartialEq, Debug)]
pub enum Ast {
    Number(i64),
    Ident(String),
    OpCall(BinOp, Box<Ast>, Box<Ast>),
    Lambda(String, Box<Ast>),
}

pub fn parser() -> impl Parser<char, Ast, Error = Simple<char>> {
    let number = just('-')
        .or_not()
        .chain::<char, _, _>(text::int(10))
        .collect::<String>()
        .from_str().unwrapped()
        .map(Ast::Number);
    let ident = text::ident().map(|ident: String| Ast::Ident(ident));

    let atomic = number.or(ident);

    let bin_op = one_of("+-").map(|c| match c {
        '+' => BinOp::Add,
        '-' => BinOp::Sub,
        _ => unreachable!()
    });

    let sum = atomic.clone()
        .then(bin_op.padded().then(atomic).repeated())
        .foldl(|lhs, (op, rhs)| Ast::OpCall(op, Box::new(lhs), Box::new(rhs)));

    let expr = sum;

    let func = just("fn")
        .ignore_then(just('('))
        .ignore_then(text::ident())
        .padded()
        .then_ignore(just("){"))
        .then(expr.padded())
        .then_ignore(just('}'))
        .map(|(name, body)| Ast::Lambda(name, Box::new(body)));

    //sum.then_ignore(end())
    func
}
