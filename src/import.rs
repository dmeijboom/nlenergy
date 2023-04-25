use std::path::PathBuf;

use anyhow::Result;
use chrono::NaiveDateTime;
use csv::ReaderBuilder;
use diesel::{insert_or_ignore_into, RunQueryDsl, SqliteConnection};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{
    energy::{Joule, Rate, State, StateList},
    models::NewHistory,
    schema::history,
};

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

pub fn cmd(mut db: SqliteConnection, filename: PathBuf) -> Result<()> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .quoting(false)
        .from_path(filename)?;
    let mut data = vec![];

    for result in reader.deserialize() {
        let record: Record = result?;
        let time = NaiveDateTime::parse_from_str(&record.time, "%Y-%m-%d %H:%M")?;

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

    for state in data {
        let checksum = state.checksum();

        insert_or_ignore_into(history::table)
            .values(&NewHistory {
                checksum: &checksum,
                rate: match state.rate {
                    Rate::Normal => true,
                    Rate::OffPeak => false,
                },
                energy: state.energy,
                time: state.time,
            })
            .execute(&mut db)?;
    }

    println!(">> imported records");

    Ok(())
}
