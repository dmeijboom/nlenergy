use diesel::{
    backend::RawValue, deserialize::FromSql, serialize::ToSql, sql_types::Bool, sqlite::Sqlite,
    AsExpression, FromSqlRow,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, AsExpression, FromSqlRow)]
#[diesel(sql_type = Bool)]
pub enum Rate {
    Normal,
    OffPeak,
}

impl FromSql<Bool, Sqlite> for Rate {
    fn from_sql(bytes: RawValue<'_, Sqlite>) -> diesel::deserialize::Result<Self> {
        Ok(match bool::from_sql(bytes)? {
            false => Self::Normal,
            true => Self::OffPeak,
        })
    }
}

impl ToSql<Bool, Sqlite> for Rate {
    fn to_sql<'a>(
        &'a self,
        out: &mut diesel::serialize::Output<'a, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        match self {
            Rate::Normal => <bool as ToSql<Bool, Sqlite>>::to_sql(&false, out),
            Rate::OffPeak => <bool as ToSql<Bool, Sqlite>>::to_sql(&true, out),
        }
    }
}
