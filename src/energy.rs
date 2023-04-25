use std::ops::{AddAssign, Sub, SubAssign};

use chrono::NaiveDateTime;
use diesel::{
    backend::{Backend, RawValue},
    deserialize::FromSql,
    serialize::ToSql,
    sql_types::{BigInt, Bool},
    AsExpression, FromSqlRow,
};
use rust_decimal::{prelude::ToPrimitive, Decimal};

#[derive(Debug, Clone, Copy, AsExpression, FromSqlRow)]
#[diesel(sql_type = BigInt)]
pub struct Joule(pub i64);

impl<DB> FromSql<BigInt, DB> for Joule
where
    DB: Backend,
    i64: FromSql<i64, DB>,
{
    fn from_sql(bytes: RawValue<'_, DB>) -> diesel::deserialize::Result<Self> {
        i64::from_sql(bytes).map(Self)
    }
}

impl<DB> ToSql<BigInt, DB> for Joule
where
    DB: Backend,
    i64: ToSql<BigInt, DB>,
{
    fn to_sql<'a>(
        &'a self,
        out: &mut diesel::serialize::Output<'a, '_, DB>,
    ) -> diesel::serialize::Result {
        self.0.to_sql(out)
    }
}

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

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, AsExpression, FromSqlRow)]
#[diesel(sql_type = Bool)]
pub enum Rate {
    Normal,
    OffPeak,
}

impl<DB> FromSql<Bool, DB> for Rate
where
    DB: Backend,
    bool: FromSql<bool, DB>,
{
    fn from_sql(bytes: RawValue<'_, DB>) -> diesel::deserialize::Result<Self> {
        Ok(match bool::from_sql(bytes)? {
            true => Self::Normal,
            false => Self::OffPeak,
        })
    }
}

impl<DB> ToSql<BigInt, DB> for Rate
where
    DB: Backend,
    bool: ToSql<Bool, DB>,
{
    fn to_sql<'a>(
        &'a self,
        out: &mut diesel::serialize::Output<'a, '_, DB>,
    ) -> diesel::serialize::Result {
        match self {
            Rate::Normal => true.to_sql(out),
            Rate::OffPeak => false.to_sql(out),
        }
    }
}

#[derive(Debug)]
pub struct State {
    pub rate: Rate,
    pub energy: Joule,
    pub time: NaiveDateTime,
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

pub trait StateList {
    fn normalize(&mut self);
}

impl StateList for Vec<State> {
    fn normalize(&mut self) {
        self.sort_by_key(|state| state.time.timestamp());
    }
}
