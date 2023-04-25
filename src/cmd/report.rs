use std::collections::HashMap;

use anyhow::Result;
use chrono::{NaiveDate, TimeZone, Utc};
use rusqlite::Connection;

use crate::energy::{Joule, Rate, State, StateList};

pub fn cmd(db: Connection, span: String) -> Result<()> {
    let (begin, end) = span.split_at(span.find("..").unwrap());
    let (begin, end) = (
        NaiveDate::parse_from_str(begin, "%Y-%m-%d")?
            .and_hms_opt(0, 0, 0)
            .map(|dt| Utc.from_utc_datetime(&dt))
            .unwrap(),
        NaiveDate::parse_from_str(&end[2..], "%Y-%m-%d")?
            .and_hms_opt(23, 59, 59)
            .map(|dt| Utc.from_utc_datetime(&dt))
            .unwrap(),
    );

    let mut nodes: HashMap<_, Vec<_>> = HashMap::new();
    let mut stmt =
        db.prepare_cached("SELECT rate, energy, time FROM history WHERE time BETWEEN ? AND ?")?;
    let iter = stmt.query_map([begin, end], |row| {
        Ok(State {
            rate: match row.get::<_, u8>(0)? {
                1 => Rate::Normal,
                2 => Rate::OffPeak,
                _ => unreachable!(),
            },
            energy: Joule(row.get(1)?),
            time: row.get(2)?,
        })
    })?;

    for state in iter {
        let state = state?;

        match nodes.get_mut(&state.rate) {
            Some(data) => data.push(state),
            None => {
                nodes.insert(state.rate, vec![state]);
            }
        }
    }

    nodes.values_mut().for_each(|data| data.normalize());

    let mut total = Joule(0);

    for (rate, data) in nodes {
        let start = data.first().map(|s| s.energy).unwrap_or(Joule(0));
        let usage = data.last().map(|s| s.energy).unwrap_or(Joule(0)) - start;

        total += usage;

        println!(
            "{}: {} kWh",
            match rate {
                Rate::Normal => "normaaltarief",
                Rate::OffPeak => "daltarief",
            },
            usage.kwh()
        );
    }

    println!("\ntotaal: {} kWh", total.kwh());

    Ok(())
}
