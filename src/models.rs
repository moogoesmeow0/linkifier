use chrono::prelude::*;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::links)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Link {
    pub link: String,
    pub created_at: chrono::NaiveDateTime,
    pub redirect: String,
}
