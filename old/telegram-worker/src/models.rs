use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use types::{Joule, Rate};

use crate::schema::usage;

#[derive(Insertable)]
#[diesel(table_name = usage)]
pub struct NewUsage {
    pub rate: Rate,
    pub delivered: Joule,
    pub received: Joule,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = usage)]
pub struct Usage {
    pub id: i32,
    pub rate: Rate,
    pub delivered: Joule,
    pub received: Joule,
    pub created_at: NaiveDateTime,
}
