use std::collections::HashMap;

use anyhow::Result;
use chrono::NaiveDate;
use sled::Db;

use crate::energy::{Joule, Rate, State, StateList};

pub fn cmd(db: Db, span: String) -> Result<()> {
    let (begin, end) = span.split_at(span.find("..").unwrap());
    let (begin, end) = (
        NaiveDate::parse_from_str(begin, "%Y-%m-%d")?,
        NaiveDate::parse_from_str(&end[2..], "%Y-%m-%d")?,
    );

    let tree = db.open_tree("energy/history")?;
    let mut nodes: HashMap<_, Vec<_>> = HashMap::new();

    for record in &tree {
        let (_, value) = record?;
        let state = State::from(value);

        if state.time.date_naive() < begin || state.time.date_naive() > end {
            continue;
        }

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
        let start = data.first().map(|s| s.energy).unwrap_or_else(|| Joule(0));
        let usage = data.last().map(|s| s.energy).unwrap_or_else(|| Joule(0)) - start;

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
