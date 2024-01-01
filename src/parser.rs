use chumsky::prelude::*;
use crate::ast::*;

pub fn parse_expr() -> impl Parser<char, Expr, Error = Simple<char>> {
    recursive(|expr| {
        let number = just('-')
            .or_not()
            .chain::<char, _, _>(text::int(10))
            .collect::<String>()
            .from_str().unwrapped()
            .map(Expr::Number);
        let ident = text::ident().map(|ident: String| Expr::Ident(ident));

        let lambda = just("fn")
            .ignore_then(just('('))
            .ignore_then(text::ident())
            .padded()
            .then_ignore(just("){"))
            .then(expr.clone().padded())
            .then_ignore(just('}'))
            .map(|(name, body)| Expr::Lambda(name, Box::new(body)));

        let funcall = ident.clone()
            .then_ignore(just('('))
            .then(expr.clone().padded())
            .then_ignore(just(')'))
            .map(|(func, arg)| Expr::FunCall(Box::new(func), Box::new(arg)));

        let parenthesized = just('(')
            .ignore_then(expr.clone())
            .then_ignore(just(')'));

        let atomic = 
            lambda
            .or(ident)
            .or(funcall)
            .or(parenthesized)
            .or(number);

        let bin_op = one_of("+-").map(|c| match c {
            '+' => BinOp::Add,
            '-' => BinOp::Sub,
            _ => unreachable!()
        });

        let sum = atomic.clone()
            .then(bin_op.padded().then(atomic.clone()).repeated())
            .foldl(|lhs, (op, rhs)| Expr::OpCall(op, Box::new(lhs), Box::new(rhs)));

        sum.or(atomic)
    })
}

pub fn parser() -> impl Parser<char, Vec<Stmt>, Error = Simple<char>> {
    let decl = just("let")
        .ignore_then(text::ident().padded())
        .then_ignore(just('=').padded())
        .then(parse_expr().padded())
        .then_ignore(just(';').padded())
        .map(|(name, body)| Stmt::Declaration(name.to_string(), body));

    decl.padded().repeated().then_ignore(end())
}
