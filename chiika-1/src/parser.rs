use crate::ast;
use chumsky::prelude::*;

pub fn ty_parser() -> impl Parser<char, ast::Ty, Error = Simple<char>> {
    recursive(|ty| {
        let params = ty
            .clone()
            .padded()
            .separated_by(just(','))
            .delimited_by(just('('), just(')'));
        let sig = params
            .then_ignore(just("->").padded())
            .then(ty.clone())
            .delimited_by(just('('), just(')'));
        let fn_ty = just("$FN")
            .ignore_then(sig)
            .map(|(param_tys, ret_ty)| ast::Ty::fun(param_tys, ret_ty));

        let raw_ty = ident_parser().map(|name| ast::Ty::Raw(name));

        fn_ty.or(raw_ty)
    })
}

pub fn append_dollar((dollar, body): (Option<char>, String)) -> String {
    format!("{}{}", if dollar.is_some() { "$" } else { "" }, body)
}

pub fn ident_parser() -> impl Parser<char, String, Error = Simple<char>> {
    just('$').or_not().then(text::ident()).map(append_dollar)
}

pub fn varref_parser() -> impl Parser<char, ast::Expr, Error = Simple<char>> {
    ident_parser().map(ast::Expr::VarRef)
}

pub fn create_funcall((func_expr, args): (ast::Expr, Vec<ast::Expr>)) -> ast::Expr {
    ast::Expr::FunCall(Box::new(func_expr), args)
}

pub fn atomic_parser(
    expr_parser: impl Parser<char, ast::Expr, Error = Simple<char>> + Clone,
) -> impl Parser<char, ast::Expr, Error = Simple<char>> {
    let number = just('-')
        .or_not()
        .chain::<char, _, _>(text::int(10))
        .collect::<String>()
        .from_str()
        .unwrapped()
        .map(ast::Expr::Number);

    let parenthesized = expr_parser.clone().delimited_by(just('('), just(')'));

    let funcall = (varref_parser().or(parenthesized.clone()))
        .then_ignore(just('('))
        .then(expr_parser.clone().padded().separated_by(just(',')))
        .then_ignore(just(')'))
        .map(create_funcall);

    funcall.or(parenthesized).or(varref_parser()).or(number)
}

pub fn expr_parser() -> impl Parser<char, ast::Expr, Error = Simple<char>> {
    recursive(|expr| {
        let bin_op = just("==")
            .or(just("<="))
            .or(just("<"))
            .or(just(">="))
            .or(just(">"))
            .or(just("+"))
            .or(just("-"));
        let sum = atomic_parser(expr.clone())
            .then(bin_op.padded().then(atomic_parser(expr.clone())).repeated())
            .foldl(|lhs, (op, rhs)| {
                ast::Expr::OpCall(op.to_string(), Box::new(lhs), Box::new(rhs))
            });

        let in_cast = atomic_parser(expr.clone())
            .then_ignore(just("as").padded())
            .then(ty_parser().padded());
        let cast = just("$CAST")
            .ignore_then(in_cast.delimited_by(just('('), just(')')))
            .map(|(expr, ty)| ast::Expr::Cast(Box::new(expr), ty));

        let alloc = just("alloc")
            .padded()
            .ignore_then(ident_parser())
            .map(ast::Expr::Alloc);

        let assign = ident_parser()
            .padded()
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .map(|(name, rhs)| ast::Expr::Assign(name, Box::new(rhs)));

        alloc.or(assign).or(cast).or(sum).or(atomic_parser(expr))
    })
}

pub fn stmts_parser() -> impl Parser<char, Vec<ast::Expr>, Error = Simple<char>> {
    expr_parser()
        .padded()
        .separated_by(just(';'))
        .allow_trailing()
}

pub fn param_parser() -> impl Parser<char, ast::Param, Error = Simple<char>> {
    ty_parser()
        .padded()
        .then(ident_parser())
        .map(|(ty, name)| ast::Param { ty, name })
}

pub fn params_parser() -> impl Parser<char, Vec<ast::Param>, Error = Simple<char>> {
    param_parser().padded().separated_by(just(','))
}

pub fn func_parser() -> impl Parser<char, ast::Function, Error = Simple<char>> {
    just("func")
        .ignore_then(ident_parser().padded())
        .then(params_parser().delimited_by(just('('), just(')')))
        .then_ignore(just("->").padded())
        .then(ty_parser().padded())
        .then(stmts_parser().padded().delimited_by(just('{'), just('}')))
        .map(|(((name, params), ret_ty), body_stmts)| ast::Function {
            name,
            params,
            ret_ty,
            body_stmts,
        })
}

pub fn extern_parser() -> impl Parser<char, ast::Extern, Error = Simple<char>> {
    just("extern")
        .ignore_then(ident_parser().padded())
        .then(params_parser().delimited_by(just('('), just(')')))
        .then_ignore(just("->").padded())
        .then(ty_parser().padded())
        .then_ignore(just(';').padded())
        .map(|((name, params), ret_ty)| ast::Extern {
            name,
            params,
            ret_ty,
        })
}

pub fn decl_parser() -> impl Parser<char, ast::Declaration, Error = Simple<char>> {
    func_parser()
        .map(ast::Declaration::Function)
        .or(extern_parser().map(ast::Declaration::Extern))
}

pub fn parser() -> impl Parser<char, Vec<ast::Declaration>, Error = Simple<char>> {
    decl_parser().padded().repeated().then_ignore(end())
}
