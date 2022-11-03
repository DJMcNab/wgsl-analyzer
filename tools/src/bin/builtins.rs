use std::io::Read;

use scraper::{Html, Selector};

type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

const HELP_STR: &str = "Usage: cargo run --bin builtins [--url https://gpuweb.github.io/gpuweb/wgsl/] [--no-prebuilt-binary]";

#[derive(Debug)]
struct Args {
    url: String,
}

fn parse_args() -> Result<Args, lexopt::Error> {
    let mut parser = lexopt::Parser::from_env();
    let mut url = None;
    while let Some(arg) = parser.next()? {
        match arg {
            lexopt::Arg::Long("help") => {
                println!("{}", HELP_STR);
                std::process::exit(0);
            }
            lexopt::Arg::Long("url") => {
                url = Some(parser.value()?.into_string()?);
            }
            _ => return Err(arg.unexpected()),
        }
    }
    Ok(Args {
        url: url.unwrap_or("https://gpuweb.github.io/gpuweb/wgsl/".to_string()),
    })
}

fn main() -> Result<()> {
    let args = parse_args()?;

    let sh = xshell::Shell::new()?;

    run(&sh, &args)?;

    Ok(())
}

struct Builtin {
    overload: String,
    description: String,
}

fn run(sh: &xshell::Shell, args: &Args) -> Result<()> {
    let mut res = reqwest::blocking::get(&args.url)?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;
    let document = Html::parse_document(&body);
    let builtin_table = Selector::parse("table.data.builtin").unwrap();
    let rows = Selector::parse("tr").unwrap();
    let columns = Selector::parse("td").unwrap();

    let mut builtins: Vec<Builtin> = vec![];

    for table in document.select(&builtin_table) {
        let mut overload = None;
        let mut description = None;
        for row in table.select(&rows) {
            let mut columns = row.select(&columns);
            let first_column = columns.next().unwrap();
            let text: String = first_column.text().collect();
            let text = text.trim().to_lowercase();
            match &*text {
                "overload" => {
                    let text_column = columns.next().unwrap();
                    let code: String = text_column.text().collect();
                    overload = Some(code);
                }
                "description" => {
                    let text_column = columns.next().unwrap();
                    let desc: String = text_column.text().collect();
                    description = Some(desc);
                }
                "parameterization" => {
                    // TODO
                }
                kind => {
                    if kind.len() > 0 {
                        eprintln!("Unknown builtin data field kind {kind}");
                        let html = table.html();
                        eprintln!("Table {html}");
                        eprintln!();
                    }
                }
            }
        }
        builtins.push(Builtin {
            overload: overload.unwrap(),
            description: description.unwrap(),
        });
    }

    Ok(())
}
