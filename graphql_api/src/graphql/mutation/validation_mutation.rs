use async_graphql::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Validation, NewValidation, CapabilityLevel};
use crate::common_utils::{UserRole, is_admin, RoleGuard};

#[derive(Default)]
pub struct ValidationMutation;

#[Object]
impl ValidationMutation {

    #[graphql(
        name = "createValidation",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    /// Creates a validation for a person's capability. Also automatically updates
    /// the associated capability's validated_level based on the average of all validations.
    pub async fn create_validation(
        &self,
        _context: &Context<'_>,
        data: NewValidation,
    ) -> Result<Validation> {
        let validation = Validation::create(&data)?;
        Ok(validation)
    }

    #[graphql(
        name = "updateValidation",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    /// Updates a validation's level. Also automatically recalculates
    /// the associated capability's validated_level.
    pub async fn update_validation(
        &self,
        _context: &Context<'_>,
        data: ValidationData,
    ) -> Result<Validation> {
        let mut validation = Validation::get_by_id(&data.id)?;

        if let Some(s) = data.validated_level {
            validation.validated_level = s;
        };

        validation.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for Validation with Option fields - only include the ones you want to update
pub struct ValidationData {
    pub id: Uuid,
    pub validated_level: Option<CapabilityLevel>,
}
