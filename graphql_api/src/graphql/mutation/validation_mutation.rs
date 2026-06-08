use async_graphql::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Validation, NewValidation, CapabilityLevel};
use crate::common_utils::{UserRole,
    is_operator, RoleGuard};

#[derive(Default)]
pub struct ValidationMutation;

// Mutation

#[Object]
impl ValidationMutation {

    #[graphql(
        name = "createValidation",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    /// An operator may create a validation of a person's capability.
    /// This records the validator's assessment of the capability and
    /// triggers a recalculation of the capability's validated_level.
    pub async fn create_validation(
        &self,
        _context: &Context<'_>,
        data: NewValidationData,
    ) -> Result<Validation> {

        let new_validation = NewValidation::new(
            data.validator_id,
            data.capability_id,
            data.validated_level,
        );

        let validation = Validation::create(&new_validation)?;

        Ok(validation)
    }

    #[graphql(
        name = "getOrCreateValidation",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    /// An operator may fetch an existing validation by validator and capability,
    /// or create a new one if none exists.
    pub async fn get_or_create_validation(
        &self,
        _context: &Context<'_>,
        data: NewValidationData,
    ) -> Result<Validation> {

        let new_validation = NewValidation::new(
            data.validator_id,
            data.capability_id,
            data.validated_level,
        );

        let validation = Validation::get_or_create(&new_validation)?;

        Ok(validation)
    }

    #[graphql(
        name = "updateValidation",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    /// An operator may update the validated_level of an existing validation.
    /// Updating a validation triggers a recalculation of the capability's validated_level.
    pub async fn update_validation(
        &self,
        _context: &Context<'_>,
        data: ValidationData,
    ) -> Result<Validation> {

        let mut validation = Validation::get_by_id(&data.id)?;

        validation.validated_level = data.validated_level;

        validation.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
/// InputObject for creating a new Validation
pub struct NewValidationData {
    pub validator_id: Uuid,
    pub capability_id: Uuid,
    pub validated_level: CapabilityLevel,
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
/// InputObject for updating an existing Validation
pub struct ValidationData {
    pub id: Uuid,
    pub validated_level: CapabilityLevel,
}
