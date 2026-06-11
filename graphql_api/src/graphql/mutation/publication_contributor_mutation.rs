use async_graphql::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{PublicationContributor, NewPublicationContributor};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct PublicationContributorMutation;

#[Object]
impl PublicationContributorMutation {

    #[graphql(
        name = "createPublicationContributor",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_publication_contributor(
        &self,
        _context: &Context<'_>,
        data: NewPublicationContributor,
    ) -> Result<PublicationContributor> {
        let publication_contributor = PublicationContributor::create(&data)?;
        Ok(publication_contributor)
    }

    #[graphql(
        name = "updatePublicationContributor",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_publication_contributor(
        &self,
        _context: &Context<'_>,
        data: PublicationContributorData,
    ) -> Result<PublicationContributor> {
        let mut publication_contributor = PublicationContributor::get_by_id(&data.id)?;

        if let Some(s) = data.contributor_role {
            publication_contributor.contributor_role = s;
        };

        publication_contributor.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for PublicationContributor with Option fields - only include the ones you want to update
pub struct PublicationContributorData {
    pub id: Uuid,
    pub contributor_role: Option<String>,
}
