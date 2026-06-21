use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use diesel::{self, ExpressionMethods, QueryDsl, Queryable, Insertable, RunQueryDsl};
use uuid::Uuid;
use async_graphql::*;

use crate::schema::record_flags;
use crate::database::connection;

/// A self-service correction request raised by a Person against their own
/// record. Operators/admins review and resolve these. Deliberately minimal.
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, SimpleObject)]
pub struct RecordFlag {
    pub id: Uuid,
    /// The Person the flag is about (the submitter's own record).
    pub person_id: Uuid,
    pub message: String,
    pub created_at: NaiveDateTime,
    /// Set when an operator/admin has actioned the flag.
    pub resolved_at: Option<NaiveDateTime>,
}

impl RecordFlag {
    pub fn create(person_id: &Uuid, message: &str) -> Result<RecordFlag> {
        let mut conn = connection()?;
        let new = NewRecordFlag {
            person_id: *person_id,
            message: message.to_owned(),
        };
        let res = diesel::insert_into(record_flags::table)
            .values(&new)
            .get_result(&mut conn)?;
        Ok(res)
    }

    /// Unresolved flags, oldest first — the operator review queue.
    pub fn get_unresolved() -> Result<Vec<RecordFlag>> {
        let mut conn = connection()?;
        let res = record_flags::table
            .filter(record_flags::resolved_at.is_null())
            .order(record_flags::created_at.asc())
            .load::<RecordFlag>(&mut conn)?;
        Ok(res)
    }

    /// Flags raised against a specific person, most recent first.
    pub fn get_by_person_id(person_id: &Uuid) -> Result<Vec<RecordFlag>> {
        let mut conn = connection()?;
        let res = record_flags::table
            .filter(record_flags::person_id.eq(person_id))
            .order(record_flags::created_at.desc())
            .load::<RecordFlag>(&mut conn)?;
        Ok(res)
    }

    pub fn resolve(id: &Uuid) -> Result<RecordFlag> {
        let mut conn = connection()?;
        let res = diesel::update(record_flags::table.filter(record_flags::id.eq(id)))
            .set(record_flags::resolved_at.eq(Some(chrono::Utc::now().naive_utc())))
            .get_result(&mut conn)?;
        Ok(res)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable)]
#[diesel(table_name = record_flags)]
pub struct NewRecordFlag {
    pub person_id: Uuid,
    pub message: String,
}
