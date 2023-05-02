use std::{path::PathBuf, time::Duration};

use anyhow::{anyhow, Result};
use async_stream::stream;
use chrono::Local;
use clap::Parser;
use diesel::{Connection, RunQueryDsl, SqliteConnection};
use dsmr5::{state::State, Reader};
use futures::{pin_mut, Stream, StreamExt};
use reqwest::Client;
use tokio::time;
use types::{Joule, Rate};

use crate::{models::NewUsage, schema::usage};

mod models;
mod schema;

#[derive(Parser)]
struct Opts {
    #[arg(env = "TELEGRAM_ENDPOINT")]
    endpoint: String,

    #[arg(short, env = "DB_PATH")]
    db_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let client = reqwest::Client::new();
    let conn = SqliteConnection::establish(opts.db_path.as_os_str().to_str().unwrap())?;

    start_loop(conn, client, opts.endpoint).await;

    Ok(())
}

impl TryFrom<State> for NewUsage {
    type Error = anyhow::Error;

    fn try_from(state: State) -> std::result::Result<Self, Self::Error> {
        let rate = match state.tariff_indicator.unwrap()[1] {
            1 => Rate::Normal,
            2 => Rate::OffPeak,
            _ => return Err(anyhow!("invalid tariff indicator")),
        };
        let reading = &state.meterreadings[rate as usize];
        let delivered = reading.to.map(Joule::from_kwh).unwrap_or_default();
        let received = reading.by.map(Joule::from_kwh).unwrap_or_default();

        Ok(NewUsage {
            rate,
            delivered,
            received,
            created_at: Local::now().naive_local(),
        })
    }
}

async fn start_loop(mut conn: SqliteConnection, client: Client, endpoint: String) {
    let stream = process_telegram_data(client, endpoint);

    pin_mut!(stream);

    let mut delivered = None;
    let mut received = None;

    while let Some(state) = stream.next().await {
        match state.and_then(TryInto::<NewUsage>::try_into) {
            Ok(new_usage) => {
                if delivered == Some(new_usage.delivered) && received == Some(new_usage.received) {
                    continue;
                }

                delivered = Some(new_usage.delivered);
                received = Some(new_usage.received);

                match diesel::insert_into(usage::table)
                    .values(new_usage)
                    .execute(&mut conn)
                {
                    Ok(_) => println!(
                        "inserted new record ({}/{})",
                        delivered.unwrap().kwh(),
                        received.unwrap().kwh()
                    ),
                    Err(err) => eprintln!("failed to insert new usage: {err:?}"),
                }
            }
            Err(err) => eprintln!("error in processing telegram data: {err:?}"),
        }
    }
}

fn process_telegram_data(
    client: Client,
    endpoint: String,
) -> impl Stream<Item = Result<State, anyhow::Error>> {
    let mut interval = time::interval(Duration::from_secs(1));

    stream! {
        loop {
            let input = client.get(&endpoint).send().await?.bytes().await?;
            let reader = Reader::new(input.into_iter());

            for readout in reader {
                let Ok(telegram) = readout.to_telegram() else {
                    continue;
                };

                let Ok(state) = Result::<State, _>::from(&telegram) else {
                    continue;
                };

                yield Ok(state);
            }

            interval.tick().await;
        }
    }
}
