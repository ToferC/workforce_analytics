// Modelled off https://github.com/clifinger/canduma/blob/master/src/user

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use diesel::{self, ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl};
use uuid::Uuid;
use async_graphql::*;

use crate::{schema::*};
use crate::common_utils::{is_admin, RoleGuard, UserRole};
use crate::models::hash_password;
use crate::database::connection;

/// A human user, linked to a Person.
pub const ACCOUNT_TYPE_HUMAN: &str = "HUMAN";
/// A non-human service account (application / data service); no Person required.
pub const ACCOUNT_TYPE_AGENT: &str = "AGENT";

/// Account lifecycle states (see docs/user_person_onboarding_lifecycle.md).
/// Created by an operator/admin; no password, cannot sign in.
pub const STATUS_PROVISIONED: &str = "PROVISIONED";
/// An activation token has been issued; awaiting redemption. Cannot sign in.
pub const STATUS_INVITED: &str = "INVITED";
/// Password set; can sign in.
pub const STATUS_ACTIVE: &str = "ACTIVE";
/// Access revoked; record retained; cannot sign in.
pub const STATUS_DISABLED: &str = "DISABLED";

/// How long an activation token is valid once issued.
const ACTIVATION_TOKEN_TTL_DAYS: i64 = 7;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserInstance {
    id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, SimpleObject, Queryable, AsChangeset)]
pub struct User {
    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub id: Uuid,
    #[graphql(skip)]
    pub hash: String,

    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub email: String,
    pub role: String,

    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub name: String,
    pub access_level: String, // AccessLevelEnum
    pub created_at: NaiveDateTime,
    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]

    pub updated_at: NaiveDateTime,
    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]

    /// Access Level: Admin
    pub access_key: String,

    #[graphql(
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    /// Access Level: Admin
    pub approved_by_user_uid: Option<Uuid>,

    /// Principal kind: "HUMAN" (default) or "AGENT". An AGENT is a non-human
    /// service account that queries the API on behalf of an application or data
    /// service. HUMAN users must be linked to a Person; ADMIN and AGENT users
    /// are exempt from that requirement.
    pub account_type: String,

    /// Account lifecycle: PROVISIONED | INVITED | ACTIVE | DISABLED. Only ACTIVE
    /// users can sign in.
    pub status: String,

    /// One-time activation token issued by `inviteUser`, cleared on activation.
    #[graphql(skip)]
    pub activation_token: Option<String>,
    #[graphql(skip)]
    pub activation_expires_at: Option<NaiveDateTime>,
}

impl User {

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let user = users::table
            .filter(users::id.eq(id))
            .get_result(&mut conn)?;

        Ok(user)
    }

    pub fn get_by_email(email: &String) -> Result<Self> {
        let mut conn = connection()?;
        let user = users::table
            .filter(users::email.eq(email))
            .get_result(&mut conn)?;

        Ok(user)
    }

    pub fn create(user: InsertableUser) -> Result<Self> {
        let mut conn = connection()?;
        let user = diesel::insert_into(users::table)
            .values(&user)
            .get_result(&mut conn)?;

        Ok(user)
    }

    pub fn get_all_ids() -> Result<Vec<Uuid>> {
        let mut conn = connection()?;
        let res = users::table
            .select(users::id)
            .load::<Uuid>(&mut conn)?;
        Ok(res)
    }

    pub fn update(&mut self) -> Result<Self> {
        let mut conn = connection()?;

        self.updated_at = chrono::Utc::now().naive_utc();

        let user = diesel::update(users::table)
            .filter(users::id.eq(&self.id))
            .set(self.clone())
            .get_result(&mut conn)?;

        Ok(user)
    }

    /// Issue (or re-issue) an activation token for a user, moving them to
    /// INVITED. Returns the token and its expiry so the caller can surface an
    /// activation link. The user must have an email to be invited.
    pub fn invite(id: &Uuid) -> Result<(String, NaiveDateTime)> {
        let mut conn = connection()?;
        let now = chrono::Utc::now().naive_utc();
        let token = Uuid::new_v4().simple().to_string();
        let expires = now + chrono::Duration::days(ACTIVATION_TOKEN_TTL_DAYS);

        diesel::update(users::table.filter(users::id.eq(id)))
            .set((
                users::activation_token.eq(Some(&token)),
                users::activation_expires_at.eq(Some(expires)),
                users::status.eq(STATUS_INVITED),
                users::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        Ok((token, expires))
    }

    /// Redeem an activation token: set the password, mark ACTIVE, and clear the
    /// token. Fails if the token is unknown or expired.
    pub fn activate(token: &str, password: &str) -> Result<Self> {
        let mut conn = connection()?;
        let now = chrono::Utc::now().naive_utc();

        let user: User = users::table
            .filter(users::activation_token.eq(token))
            .get_result(&mut conn)
            .map_err(|_| Error::new("Invalid or already-used activation token"))?;

        if let Some(expiry) = user.activation_expires_at {
            if expiry < now {
                return Err(Error::new("Activation token has expired"));
            }
        }

        let hash = crate::models::hash_password(password)
            .map_err(|_| Error::new("Unable to hash password"))?
            .to_string();

        let updated = diesel::update(users::table.filter(users::id.eq(&user.id)))
            .set((
                users::hash.eq(hash),
                users::status.eq(STATUS_ACTIVE),
                users::activation_token.eq::<Option<String>>(None),
                users::activation_expires_at.eq::<Option<NaiveDateTime>>(None),
                users::updated_at.eq(now),
            ))
            .get_result(&mut conn)?;

        Ok(updated)
    }

    /// Set a user's lifecycle status directly (e.g. disable/enable).
    pub fn set_status(id: &Uuid, status: &str) -> Result<Self> {
        let mut conn = connection()?;
        let updated = diesel::update(users::table.filter(users::id.eq(id)))
            .set((
                users::status.eq(status),
                users::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .get_result(&mut conn)?;
        Ok(updated)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable)]
#[diesel(table_name = users)]
pub struct InsertableUser {
    pub hash: String,
    pub email: String,
    pub role: String,
    pub name: String,
    pub access_level: String, // AccessLevelEnum
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub access_key: String,
    pub approved_by_user_uid: Option<Uuid>,
    pub account_type: String,
    pub status: String,
    pub activation_token: Option<String>,
    pub activation_expires_at: Option<NaiveDateTime>,
}

impl InsertableUser {
    /// Build a PROVISIONED human account with no usable password — the form an
    /// operator-created account takes until the person activates it. Email is
    /// the future login id and invite target (required).
    pub fn provisioned(email: &str, name: &str) -> Self {
        let now = chrono::Utc::now().naive_utc();
        InsertableUser {
            hash: "".to_owned(),
            email: email.to_owned(),
            role: "USER".to_owned(),
            name: name.to_owned(),
            access_level: "detailed".to_owned(),
            created_at: now,
            updated_at: now,
            access_key: "".to_owned(),
            approved_by_user_uid: None,
            account_type: ACCOUNT_TYPE_HUMAN.to_owned(),
            status: STATUS_PROVISIONED.to_owned(),
            activation_token: None,
            activation_expires_at: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, InputObject)]
/// Input Struct to create a new user. Only accessible by Administrators.
pub struct UserData {
    pub name: String,
    pub email: String,
    pub password: String,
    /// UserRole in system: USER, OPERATOR, ANALYST, ADMIN
    pub role: String,
    /// Principal kind: "HUMAN" (default) or "AGENT". Agents are non-human
    /// service accounts and are not required to be linked to a Person.
    pub account_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, InputObject)]
/// Input Struct to create a new user. Only accessible by Administrators.
pub struct UserUpdate {
    pub id: Uuid,
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    /// UserRole in system: USER, OPERATOR, ANALYST, ADMIN
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SimpleObject)]
pub struct SlimUser {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub access_level: String,
}

#[derive(Shrinkwrap, Clone, Default)]
pub struct LoggedUser(pub Option<SlimUser>);

impl From<SlimUser> for LoggedUser {
    fn from(slim_user: SlimUser) -> Self {
        LoggedUser(Some(slim_user))
    }
}

impl From<UserData> for InsertableUser {
    fn from(user_data: UserData) -> Self {

        let updated_at = chrono::Utc::now().naive_utc();

        let UserData {
            name,
            email,
            password,
            role,
            account_type,
        } = user_data;

        let hash = hash_password(&password)
            .expect("Unable to hash password")
            .to_string();

        Self {
            email,
            hash,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at,
            name,
            role,
            access_key: "".to_owned(),
            access_level: "detailed".to_owned(),
            approved_by_user_uid: None,
            account_type: account_type.unwrap_or_else(|| ACCOUNT_TYPE_HUMAN.to_owned()),
            // Admin-created accounts come with a password, so they are usable
            // immediately. The provisioning path uses InsertableUser::provisioned.
            status: STATUS_ACTIVE.to_owned(),
            activation_token: None,
            activation_expires_at: None,
        }
    }
}

impl From<User> for SlimUser {
    fn from(user: User) -> Self {
        let User {
            id,
            email,
            role,
            access_level,
            ..
        } = user;

        Self {
            id,
            email,
            role,
            access_level,
        }
    }
}

#[derive(Debug, Deserialize, InputObject)]
pub struct LoginQuery {
    pub email: String,
    pub password: String,
}