use std::path::PathBuf;

use anyhow::Result;
use chrono::{Local, TimeZone};
use csv::ReaderBuilder;
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::Deserialize;

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

pub fn cmd(db: Connection, filename: PathBuf) -> Result<()> {
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

    let mut stmt = db.prepare(
        "INSERT OR IGNORE INTO history (checksum, time, rate, energy) VALUES (?, ?, ?, ?)",
    )?;

    for state in data {
        let checksum = state.checksum();

        stmt.execute((
            checksum,
            state.time.timestamp(),
            state.rate as u8,
            state.energy.0,
        ))?;
    }

    println!(">> imported records");

    Ok(())
}
