use async_graphql::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Capability, CapabilityLevel, NewCapability, Person, RecordFlag};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct SelfServiceMutation;

/// Fields a person may change on their own record. All optional; omitted fields
/// are left unchanged.
#[derive(Debug, Deserialize, Serialize, InputObject)]
pub struct MyPersonUpdate {
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub work_address: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[Object]
impl SelfServiceMutation {
    /// Update the caller's own Person record (contact details). Authorized by
    /// ownership: the record must be linked to the caller's user account, so no
    /// role guard is needed — a plain authenticated user may edit only themself.
    #[graphql(name = "updateMyPerson")]
    pub async fn update_my_person(
        &self,
        ctx: &Context<'_>,
        data: MyPersonUpdate,
    ) -> Result<Person> {
        let uid = ctx
            .data_opt::<Uuid>()
            .copied()
            .ok_or_else(|| Error::new("Not authenticated"))?;

        let mut person = Person::get_by_user_id(&uid)
            .map_err(|_| Error::new("No person record is linked to your account"))?;

        if let Some(s) = data.family_name { person.family_name = s; }
        if let Some(s) = data.given_name { person.given_name = s; }
        if let Some(s) = data.email { person.email = s; }
        if let Some(s) = data.phone { person.phone = s; }
        if let Some(s) = data.work_address { person.work_address = s; }
        if let Some(s) = data.city { person.city = s; }
        if let Some(s) = data.province { person.province = s; }
        if let Some(s) = data.postal_code { person.postal_code = s; }
        if let Some(s) = data.country { person.country = s; }

        person.update()
    }

    /// Self-declare a capability on the caller's own Person record. Authorized
    /// by ownership like updateMyPerson: the capability is created with only a
    /// self-identified level — validation remains an admin action, so the trust
    /// model is unchanged.
    #[graphql(name = "addMyCapability")]
    pub async fn add_my_capability(
        &self,
        ctx: &Context<'_>,
        skill_id: Uuid,
        self_identified_level: CapabilityLevel,
    ) -> Result<Capability> {
        let uid = ctx
            .data_opt::<Uuid>()
            .copied()
            .ok_or_else(|| Error::new("Not authenticated"))?;

        let person = Person::get_by_user_id(&uid)
            .map_err(|_| Error::new("No person record is linked to your account"))?;

        // One capability per person per skill: re-declaring updates the
        // self-identified level instead of duplicating the row.
        if let Ok(mut existing) = Capability::get_by_person_and_skill(&person.id, &skill_id) {
            existing.self_identified_level = self_identified_level;
            return existing.update();
        }

        let new_capability = NewCapability::new(
            person.id,
            skill_id,
            person.organization_id,
            self_identified_level,
        );

        Capability::create(&new_capability)
    }

    /// Raise a correction flag against the caller's own record for operators to
    /// review.
    #[graphql(name = "flagRecordIssue")]
    pub async fn flag_record_issue(
        &self,
        ctx: &Context<'_>,
        message: String,
    ) -> Result<RecordFlag> {
        let uid = ctx
            .data_opt::<Uuid>()
            .copied()
            .ok_or_else(|| Error::new("Not authenticated"))?;

        if message.trim().is_empty() {
            return Err(Error::new("Flag message cannot be empty"));
        }

        let person = Person::get_by_user_id(&uid)
            .map_err(|_| Error::new("No person record is linked to your account"))?;

        RecordFlag::create(&person.id, message.trim())
    }

    /// Mark a record flag resolved. Operator/admin only.
    #[graphql(
        name = "resolveRecordFlag",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn resolve_record_flag(
        &self,
        _ctx: &Context<'_>,
        id: Uuid,
    ) -> Result<RecordFlag> {
        RecordFlag::resolve(&id)
    }
}
