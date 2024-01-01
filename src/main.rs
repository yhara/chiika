mod ast;
mod parser;
use chumsky::Parser;
use ariadne::{Label, Report, ReportKind, Source};
use parser::parser;

fn print_error(src: &str, span: std::ops::Range<usize>, msg: String) {
    Report::build(ReportKind::Error, "", span.start)
        .with_message(msg.clone())
        .with_label(
            Label::new(("", span))
                .with_message(msg))
        .finish()
        .print(("", Source::from(src)))
        .unwrap();
}

fn main() {
    let src = "
        let a = 1;
        let b = 2;
        let f = fn(x){ x };
        let _ = f(a);
        ";
    match parser().parse(src) {
        Ok(ast) => {
            dbg!(&ast);
        }
        Err(errs) => errs.into_iter().for_each(|e| {
            print_error(src, e.span(), e.to_string())
        })
    };
}
