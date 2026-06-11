use async_graphql::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{LanguageData, NewLanguageData, LanguageName, LanguageLevel};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct LanguageMutation;

#[Object]
impl LanguageMutation {

    #[graphql(
        name = "createLanguageData",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_language_data(
        &self,
        _context: &Context<'_>,
        data: NewLanguageData,
    ) -> Result<LanguageData> {
        let language_data = LanguageData::create(&data)?;
        Ok(language_data)
    }

    #[graphql(
        name = "updateLanguageData",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_language_data(
        &self,
        _context: &Context<'_>,
        data: LanguageDataUpdate,
    ) -> Result<LanguageData> {
        let mut language_data = LanguageData::get_by_id(&data.id)?;

        if let Some(s) = data.language_name {
            language_data.language_name = s;
        };

        if let Some(s) = data.reading {
            language_data.reading = Some(s);
        };

        if let Some(s) = data.writing {
            language_data.writing = Some(s);
        };

        if let Some(s) = data.speaking {
            language_data.speaking = Some(s);
        };

        language_data.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for LanguageData with Option fields - only include the ones you want to update
pub struct LanguageDataUpdate {
    pub id: Uuid,
    pub language_name: Option<LanguageName>,
    pub reading: Option<LanguageLevel>,
    pub writing: Option<LanguageLevel>,
    pub speaking: Option<LanguageLevel>,
}
