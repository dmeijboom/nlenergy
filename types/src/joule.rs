use std::ops::{AddAssign, Sub, SubAssign};

use diesel::{
    backend::RawValue, deserialize::FromSql, serialize::ToSql, sql_types::BigInt, sqlite::Sqlite,
    AsExpression, FromSqlRow,
};

#[derive(Debug, Default, PartialEq, Clone, Copy, AsExpression, FromSqlRow)]
#[diesel(sql_type = BigInt)]
pub struct Joule(pub i64);

impl Joule {
    #[inline]
    pub fn kwh(self) -> f64 {
        (self.0 as f64) / 3600000.0
    }

    #[inline]
    pub fn from_kwh(kwh: f64) -> Self {
        Self((kwh * 3600000.0).round() as i64)
    }
}

impl FromSql<BigInt, Sqlite> for Joule {
    fn from_sql(bytes: RawValue<'_, Sqlite>) -> diesel::deserialize::Result<Self> {
        i64::from_sql(bytes).map(Self)
    }
}

impl ToSql<BigInt, Sqlite> for Joule {
    fn to_sql<'a>(
        &'a self,
        out: &mut diesel::serialize::Output<'a, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        <i64 as ToSql<BigInt, Sqlite>>::to_sql(&self.0, out)
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
