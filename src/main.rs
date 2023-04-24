#![feature(drain_filter)]

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod energy;
mod import;
mod report;

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    cmd: Cmd,
}

#[derive(Parser)]
enum Cmd {
    #[clap(name = "import")]
    Import {
        #[arg(short)]
        filename: PathBuf,
    },

    #[clap(name = "report")]
    Report { span: String },
}

fn main() -> Result<()> {
    let db = sled::open("_data")?;
    let opts = Opts::parse();

    match opts.cmd {
        Cmd::Import { filename } => import::cmd(db, filename),
        Cmd::Report { span } => report::cmd(db, span),
    }
}
