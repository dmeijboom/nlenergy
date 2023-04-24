use std::path::PathBuf;

use anyhow::Result;
use chrono::{Local, TimeZone};
use csv::ReaderBuilder;
use rust_decimal::Decimal;
use serde::Deserialize;
use sled::Db;

use crate::energy::{Joule, Rate, State, StateList};

#[derive(Debug, Deserialize)]
struct Record {
    time: String,
    #[serde(rename = "Electricity imported T1")]
    t1_import: Decimal,
    #[serde(rename = "Electricity imported T2")]
    t2_import: Decimal,
    #[serde(rename = "Electricity exported T1")]
    t1_export: Decimal,
    #[serde(rename = "Electricity exported T2")]
    t2_export: Decimal,
}

pub fn cmd(db: Db, filename: PathBuf) -> Result<()> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .quoting(false)
        .from_path(filename)?;
    let mut data = vec![];

    for result in reader.deserialize() {
        let record: Record = result?;
        let time = Local.datetime_from_str(&record.time, "%Y-%m-%d %H:%M")?;

        data.push(State {
            rate: Rate::Normal,
            energy: Joule::from_kwh(record.t1_import - record.t1_export).unwrap(),
            time,
        });

        data.push(State {
            rate: Rate::OffPeak,
            energy: Joule::from_kwh(record.t2_import - record.t2_export).unwrap(),
            time,
        });
    }

    data.normalize();

    let mut count = 0;
    let tree = db.open_tree("energy/history")?;

    for state in data {
        let checksum = state.checksum();

        if tree.contains_key(&checksum)? {
            continue;
        }

        count += 1;
        tree.insert::<_, Vec<u8>>(checksum, (&state).into())?;
    }

    println!(">> imported {count} records");

    Ok(())
}
