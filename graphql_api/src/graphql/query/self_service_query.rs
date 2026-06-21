use async_graphql::*;
use uuid::Uuid;

use crate::models::{User, Person, RecordFlag};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct SelfServiceQuery;

/// The authenticated caller's account and linked person record. Drives the
/// self-service "My profile" experience.
#[derive(SimpleObject)]
pub struct Me {
    pub user: User,
    /// The Person linked to this account, if any (admins/agents may have none).
    pub person: Option<Person>,
}

#[Object]
impl SelfServiceQuery {
    /// The currently authenticated user and their linked Person.
    #[graphql(name = "me")]
    pub async fn me(&self, ctx: &Context<'_>) -> Result<Me> {
        let uid = ctx
            .data_opt::<Uuid>()
            .copied()
            .ok_or_else(|| Error::new("Not authenticated"))?;
        let user = User::get_by_id(&uid)?;
        let person = Person::get_by_user_id(&uid).ok();
        Ok(Me { user, person })
    }

    /// Unresolved record-correction flags, oldest first — the operator review
    /// queue. Operator/admin only.
    #[graphql(
        name = "recordFlags",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn record_flags(&self, _ctx: &Context<'_>) -> Result<Vec<RecordFlag>> {
        RecordFlag::get_unresolved()
    }
}
