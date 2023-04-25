#![feature(drain_filter)]

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use diesel::{Connection, SqliteConnection};

mod energy;
mod import;
mod models;
mod report;
mod schema;
mod telegram;

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

    #[clap(name = "telegram")]
    Telegram { endpoint: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = SqliteConnection::establish("./data.db")?;
    let conn = rusqlite::Connection::open("data.db")?;
    let opts = Opts::parse();

    match opts.cmd {
        Cmd::Import { filename } => import::cmd(db, filename),
        Cmd::Report { span } => report::cmd(conn, span),
        Cmd::Telegram { endpoint } => telegram::cmd(conn, endpoint).await,
    }
}
