use std::ops::{AddAssign, Sub, SubAssign};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use rust_decimal::{prelude::ToPrimitive, Decimal};

#[derive(Debug, Clone, Copy)]
pub struct Joule(pub i64);

impl Joule {
    #[inline]
    pub fn kwh(self) -> Decimal {
        Decimal::new(self.0, 0) / Decimal::new(3600, 0) / Decimal::new(1000, 0)
    }

    #[inline]
    pub fn from_kwh(kwh: Decimal) -> Option<Self> {
        (kwh * Decimal::new(3600, 0) * Decimal::new(1000, 0))
            .to_i64()
            .map(Self)
    }
}

impl Sub for Joule {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl AddAssign for Joule {
    fn add_assign(&mut self, rhs: Self) {
        *self = Joule(self.0 + rhs.0);
    }
}

impl SubAssign for Joule {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Joule(self.0 - rhs.0);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Rate {
    Normal = 1,
    OffPeak = 2,
}

#[derive(Debug)]
pub struct State {
    pub rate: Rate,
    pub energy: Joule,
    pub time: DateTime<Local>,
}

impl State {
    /// checksum returns a SHA256 checksum of the rate + energy
    pub fn checksum(&self) -> String {
        let b: Vec<u8> = self.into();
        sha256::digest(&b[..b.len() - 8])
    }
}

impl From<&State> for Vec<u8> {
    fn from(value: &State) -> Self {
        let kind_bytes = value.rate as u8;
        let energy_bytes = value.energy.0.to_le_bytes();
        let time_bytes = value.time.timestamp().to_le_bytes();

        [
            &[kind_bytes],
            energy_bytes.as_slice(),
            time_bytes.as_slice(),
        ]
        .concat()
    }
}

impl From<Vec<u8>> for State {
    fn from(value: Vec<u8>) -> Self {
        let kind_bytes = value[0];
        let energy_bytes: [u8; 8] = value[1..9].try_into().unwrap();
        let time_bytes: [u8; 8] = value[9..].try_into().unwrap();

        let rate = match kind_bytes {
            1 => Rate::Normal,
            2 => Rate::OffPeak,
            _ => unreachable!("invalid rate"),
        };
        let energy = Joule(i64::from_le_bytes(energy_bytes));
        let time = Local.from_utc_datetime(
            &NaiveDateTime::from_timestamp_opt(i64::from_le_bytes(time_bytes), 0).unwrap(),
        );

        Self { rate, energy, time }
    }
}

pub trait StateList {
    fn normalize(&mut self);
}

impl StateList for Vec<State> {
    fn normalize(&mut self) {
        self.sort_by_key(|state| state.time.with_timezone(&Utc).timestamp());
    }
}
