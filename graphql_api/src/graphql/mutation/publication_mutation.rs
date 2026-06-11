use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Publication, NewPublication, PublicationStatus};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct PublicationMutation;

#[Object]
impl PublicationMutation {

    #[graphql(
        name = "createPublication",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_publication(
        &self,
        _context: &Context<'_>,
        data: NewPublication,
    ) -> Result<Publication> {
        let publication = Publication::create(&data)?;
        Ok(publication)
    }

    #[graphql(
        name = "updatePublication",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_publication(
        &self,
        _context: &Context<'_>,
        data: PublicationData,
    ) -> Result<Publication> {
        let mut publication = Publication::get_by_id(&data.id)?;

        if let Some(s) = data.title {
            publication.title = s;
        };

        if let Some(s) = data.subject_text {
            publication.subject_text = s;
        };

        if let Some(s) = data.publication_status {
            publication.publication_status = s;
        };

        if let Some(s) = data.url_string {
            publication.url_string = Some(s);
        };

        if let Some(s) = data.publishing_id {
            publication.publishing_id = Some(s);
        };

        if let Some(s) = data.submitted_date {
            publication.submitted_date = Some(s);
        };

        if let Some(s) = data.published_datestamp {
            publication.published_datestamp = Some(s);
        };

        publication.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for Publication with Option fields - only include the ones you want to update
pub struct PublicationData {
    pub id: Uuid,
    pub title: Option<String>,
    pub subject_text: Option<String>,
    pub publication_status: Option<PublicationStatus>,
    pub url_string: Option<String>,
    pub publishing_id: Option<String>,
    pub submitted_date: Option<NaiveDateTime>,
    pub published_datestamp: Option<NaiveDateTime>,
}
