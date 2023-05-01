use std::time::Duration;

use anyhow::Result;
use async_stream::stream;
use clap::Parser;
use dsmr5::{state::State, Reader};
use futures::{pin_mut, Stream, StreamExt};
use reqwest::Client;
use tokio::time;

#[derive(Parser)]
struct Opts {
    #[arg(env = "TELEGRAM_ENDPOINT")]
    endpoint: String,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let client = reqwest::Client::new();

    start_loop(client, opts.endpoint).await;
}

async fn start_loop(client: Client, endpoint: String) {
    let stream = process_telegram_data(client, endpoint);

    pin_mut!(stream);

    while let Some(state) = stream.next().await {
        match state {
            Ok(state) => println!("{:?}", state),
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
