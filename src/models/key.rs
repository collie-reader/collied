use chrono::{DateTime, FixedOffset};
use rusqlite::Row;
use sea_query::{Expr, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::DbState;

use super::database::Keys;

#[derive(Serialize)]
pub struct Key {
    pub id: i32,
    pub access: String,
    pub secret: String,
    pub description: Option<String>,
    pub expired_at: Option<DateTime<FixedOffset>>,
}

impl From<&Row<'_>> for Key {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get_unwrap("id"),
            access: row.get_unwrap("access"),
            secret: row.get_unwrap("secret"),
            description: row.get_unwrap("description"),
            expired_at: row.get_unwrap("expired_at"),
        }
    }
}

#[derive(Deserialize)]
pub struct KeyToCreate {
    pub access: String,
    pub secret: String,
    pub description: Option<String>,
    pub expired_at: Option<DateTime<FixedOffset>>,
}

pub fn create(db_state: &DbState, arg: &KeyToCreate) -> Result<usize> {
    let (sql, values) = Query::insert()
        .into_table(Keys::Table)
        .columns([
            Keys::Access,
            Keys::Secret,
            Keys::Description,
            Keys::ExpiredAt,
        ])
        .values_panic([
            (*arg.access).into(),
            (*arg.secret).into(),
            arg.description.clone().into(),
            arg.expired_at.into(),
        ])
        .build_rusqlite(SqliteQueryBuilder);

    let db = db_state.db.lock().unwrap();
    Ok(db.execute(sql.as_str(), &*values.as_params())?)
}

pub fn find_secret_by_access(db_state: &DbState, access_key: &str) -> Result<String> {
    let (sql, values) = Query::select()
        .columns([Keys::Secret])
        .from(Keys::Table)
        .and_where(Expr::col(Keys::Access).eq(access_key))
        .limit(1)
        .build_rusqlite(SqliteQueryBuilder);

    let db = db_state.db.lock().unwrap();
    let mut stmt = db.prepare(sql.as_str())?;
    let mut rows = stmt.query(&*values.as_params())?;

    match rows.next()?.map(Key::from) {
        Some(key) => Ok(key.secret),
        None => Err(Error::Unauthorized),
    }
}
