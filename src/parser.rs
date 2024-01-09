use crate::ast;
use chumsky::prelude::*;

pub fn expr_parser() -> impl Parser<char, ast::Expr, Error = Simple<char>> {
    recursive(|expr| {
        let number = just('-')
            .or_not()
            .chain::<char, _, _>(text::int(10))
            .collect::<String>()
            .from_str()
            .unwrapped()
            .map(ast::Expr::Number);
        let ident = text::ident().map(|ident: String| ast::Expr::Ident(ident));

        let funcall = text::ident()
            .then_ignore(just('('))
            .then(expr.clone().padded())
            .then_ignore(just(')'))
            .map(|(func_name, arg)| ast::Expr::FunCall(func_name, Box::new(arg)));

        let parenthesized = just('(').ignore_then(expr.clone()).then_ignore(just(')'));

        let atomic = funcall.or(ident).or(parenthesized).or(number);

        let bin_op = one_of("+-").map(|c| match c {
            '+' => ast::BinOp::Add,
            '-' => ast::BinOp::Sub,
            _ => unreachable!(),
        });

        let sum = atomic
            .clone()
            .then(bin_op.padded().then(atomic.clone()).repeated())
            .foldl(|lhs, (op, rhs)| ast::Expr::OpCall(op, Box::new(lhs), Box::new(rhs)));

        sum.or(atomic)
    })
}

pub fn stmts_parser() -> impl Parser<char, Vec<ast::Expr>, Error = Simple<char>> {
    expr_parser()
        .padded()
        .separated_by(just(';'))
        .allow_trailing()
}

pub fn func_parser() -> impl Parser<char, ast::Function, Error = Simple<char>> {
    just("func")
        .ignore_then(text::ident().padded())
        .then_ignore(just('(').padded())
        .then(text::ident().padded())
        .then_ignore(just(')').padded())
        .then_ignore(just('{').padded())
        .then(stmts_parser())
        .then_ignore(just('}').padded())
        .map(|((name, arg_name), body_stmts)| ast::Function {
            name,
            arg_name,
            body_stmts,
        })
}

pub fn parser() -> impl Parser<char, Vec<ast::Function>, Error = Simple<char>> {
    func_parser().padded().repeated().then_ignore(end())
}
