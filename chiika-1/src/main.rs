mod ast;
mod codegen;
mod parser;
use anyhow::{bail, Result, Context};
use ariadne::{Label, Report, ReportKind, Source};
use chumsky::Parser;
use parser::parser;

fn render_parse_error(src: &str, span: std::ops::Range<usize>, msg: String) -> String {
    let mut rendered = vec![];
    Report::build(ReportKind::Error, "", span.start)
        .with_message(msg.clone())
        .with_label(Label::new(("", span)).with_message(msg))
        .finish()
        .write(("", Source::from(src)), &mut rendered)
        .unwrap();
    String::from_utf8_lossy(&rendered).to_string()
}

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let Some(path) = args.get(1) else {
        bail!("usage: chiika-1 a.chiika1");
    };
    let src = std::fs::read_to_string(path)
        .context(format!("failed to read {}", path))?;
    let ast = match parser().parse(src) {
        Ok(x) => x,
        Err(errs) => {
            let src = std::fs::read_to_string(path)?;
            let mut s = String::new();
            errs.into_iter().for_each(|e| {
                s += &render_parse_error(&src, e.span(), e.to_string());
            });
            bail!(s);
        }
    };
    //dbg!(&ast);
    codegen::run(ast)
}
