use chrono::NaiveDateTime;
use diesel::Insertable;

use crate::{energy::Joule, schema::history};

#[derive(Insertable)]
#[diesel(table_name = history)]
pub struct NewHistory<'a> {
    pub checksum: &'a str,
    pub rate: bool,
    pub energy: Joule,
    pub time: NaiveDateTime,
}
