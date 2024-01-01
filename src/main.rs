mod ast;
mod parser;
use chumsky::Parser;
use ariadne::{Label, Report, ReportKind, Source};
use parser::parser;

fn print_parse_error(src: &str, span: std::ops::Range<usize>, msg: String) {
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
        func foo(x) {
          print(x+x);
        }
        func main(_) {
          foo(1);
        }
        ";
    let ast = match parser().parse(src) {
        Ok(x) => x,
        Err(errs) => {
            errs.into_iter().for_each(|e| {
                print_parse_error(src, e.span(), e.to_string());
            });
            return;
        }
    };
    dbg!(&ast);
}
