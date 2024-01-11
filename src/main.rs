mod ast;
mod codegen;
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
        extern sleep($ENV $env, $FN(($ENV, int) -> $FUTURE) $cont, int n) -> $FUTURE;
        extern chiika_env_push($ENV $env, any obj) -> int;
        extern chiika_env_pop($ENV $env) -> any;
        extern chiika_start_tokio(int n) -> int;

        func foo($ENV $env, $FN((int) -> $FUTURE) $cont) -> $FUTURE {
          chiika_env_push($env, $cont);
          print(100);
          sleep($env, foo_1, 1)
        }
        func foo_1($ENV $env, int _) -> $FUTURE {
          print(200);
          ($CAST(chiika_env_pop($env) as $FN(($ENV, int) -> $FUTURE)))($env, 0)
        }
        func chiika_main($ENV $env, $FN((int) -> $FUTURE) $cont) -> $FUTURE {
          foo($env, $cont)
        }
        func main() -> int {
          chiika_start_tokio(0);
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
    //dbg!(&ast);
    codegen::run(ast)
}
