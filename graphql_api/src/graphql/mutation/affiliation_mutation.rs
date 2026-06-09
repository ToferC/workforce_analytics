use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Affiliation, NewAffiliation};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct AffiliationMutation;

#[Object]
impl AffiliationMutation {

    #[graphql(
        name = "createAffiliation",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_affiliation(
        &self,
        _context: &Context<'_>,
        data: NewAffiliation,
    ) -> Result<Affiliation> {
        let affiliation = Affiliation::create(&data)?;
        Ok(affiliation)
    }

    #[graphql(
        name = "updateAffiliation",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_affiliation(
        &self,
        _context: &Context<'_>,
        data: AffiliationData,
    ) -> Result<Affiliation> {
        let mut affiliation = Affiliation::get_by_id(&data.id)?;

        if let Some(s) = data.affiliation_role {
            affiliation.affiliation_role = s;
        };

        if let Some(s) = data.start_datestamp {
            affiliation.start_datestamp = s;
        };

        if let Some(s) = data.end_date {
            affiliation.end_date = Some(s);
        };

        affiliation.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for Affiliation with Option fields - only include the ones you want to update
pub struct AffiliationData {
    pub id: Uuid,
    pub affiliation_role: Option<String>,
    pub start_datestamp: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
}
