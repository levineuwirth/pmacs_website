#![forbid(unsafe_code)]

use pmacs_site::docs::DocsSource;
use pmacs_site::{Result, check, serve, site};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
usage: pmacs-site <command> [options]

Run from the repository root.

commands:
  build    render the site into the output directory
  serve    build, then serve the output on 127.0.0.1
  check    verify that every local link in the output resolves

options:
  --out <dir>      output directory (default: _site)
  --pmacs <path>   read pmacs's docs from this local repository instead of
                   the cached clone of the configured remote
  --no-docs        skip the release docs and the history
  --port <n>       port for serve (default: 8000)
";

struct Args {
    command: String,
    out: PathBuf,
    pmacs: Option<PathBuf>,
    no_docs: bool,
    port: u16,
}

fn parse(args: Vec<String>) -> Result<Args> {
    let mut it = args.into_iter();
    let command = it.next().unwrap_or_else(|| "help".into());
    let mut a = Args {
        command,
        out: PathBuf::from("_site"),
        pmacs: None,
        no_docs: false,
        port: 8000,
    };
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--out" => a.out = PathBuf::from(value()?),
            "--pmacs" => a.pmacs = Some(PathBuf::from(value()?)),
            "--no-docs" => a.no_docs = true,
            "--port" => a.port = value()?.parse().map_err(|_| "--port needs a number")?,
            other => return Err(format!("unknown option `{other}`\n\n{USAGE}").into()),
        }
    }
    Ok(a)
}

fn run(args: Vec<String>) -> Result<()> {
    let a = parse(args)?;
    let root = std::env::current_dir()?;
    let docs = match (a.no_docs, a.pmacs) {
        (true, _) => DocsSource::None,
        (false, Some(path)) => DocsSource::Local(path),
        (false, None) => DocsSource::Remote,
    };
    let opts = site::BuildOptions {
        root,
        out: a.out.clone(),
        docs,
    };
    match a.command.as_str() {
        "build" => {
            println!("{}", site::build(&opts)?);
            Ok(())
        }
        "serve" => {
            println!("{}", site::build(&opts)?);
            serve::serve(&a.out, a.port)
        }
        "check" => check::run(&a.out),
        "help" | "--help" | "-h" => {
            print!("{USAGE}");
            Ok(())
        }
        other => Err(format!("unknown command `{other}`\n\n{USAGE}").into()),
    }
}

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("pmacs-site: {e}");
            ExitCode::FAILURE
        }
    }
}
