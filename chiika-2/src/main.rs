mod ast;
mod compiler;
mod parser;
use anyhow::{bail, Result};
use ariadne::{Label, Report, ReportKind, Source};
use chumsky::Parser;
use parser::parser;

fn print_parse_error(src: &str, span: std::ops::Range<usize>, msg: String) {
    Report::build(ReportKind::Error, "", span.start)
        .with_message(msg.clone())
        .with_label(Label::new(("", span)).with_message(msg))
        .finish()
        .print(("", Source::from(src)))
        .unwrap();
}

fn main() -> Result<()> {
    let src = "
      extern print(int n) -> int;
      extern_async sleep(int n) -> int;
      fun foo() -> int {
        print(100);
        print(sleep(1));
        print(200);
        300
      }
      fun chiika_main() -> int {
        print(foo());
        0
      }
    ";
    let ast = match parser().parse(src) {
        Ok(x) => x,
        Err(errs) => {
            errs.into_iter().for_each(|e| {
                print_parse_error(src, e.span(), e.to_string());
            });
            bail!("");
        }
    };
    let compiled = compiler::compile(ast);
    println!("{}", ast::to_source(compiled));
    Ok(())
}
