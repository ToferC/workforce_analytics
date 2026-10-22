use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use serde_json::json;

use crate::models::{Role, RoleAssignment, NewRole, Person, PersonnelType, AuditEvent, MilitaryOccupation, Rank, OccupationalGroup};
use crate::common_utils::{UserRole,
    is_operator, RoleGuard};
use crate::graphql::authz;
// use rdkafka::producer::FutureProducer;
// use crate::kafka::send_message;

#[derive(Default)]
pub struct RoleMutation;

// Mutation Example

#[Object]
impl RoleMutation {

    #[graphql(
        name = "createRole", 
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_role(
        &self,
        context: &Context<'_>,
        role_data: NewRole,
    ) -> Result<Role> {

        authz::require_manage_team(context, &role_data.team_id)?;

        // If the role is assigned to a person, ensure the HR fields
        // provided match the person's personnel type
        if let Some(person_id) = role_data.person_id {
            require_classification_for(
                &person_id,
                &RoleClassification {
                    military_occupation: role_data.military_occupation,
                    rank: role_data.rank,
                    occupational_group: role_data.occupational_group,
                    occupational_level: role_data.occupational_level,
                },
            )?;
        }

        // A new role with an explicit reporting line must report to a more
        // senior position (same rule as setRoleReportsTo).
        if let Some(manager_id) = role_data.reports_to {
            Role::check_create_reports_to(
                &role_data.team_id,
                role_data.rank,
                role_data.occupational_level,
                &manager_id,
            )?;
        }

        let role = Role::create(&role_data)?;

        // A person holds one active role at a time. If this new role was
        // created already occupied, close the incumbent's tenure on any other
        // role (recording it as career history) and vacate those positions.
        if let Some(person_id) = role.person_id {
            RoleAssignment::close_others_for_person(&person_id, &role.id)?;
        }

        AuditEvent::log(
            context,
            "role.create",
            "role",
            Some(role.id),
            format!("Created role “{}”", role.title_en),
            Some(json!({
                "team_id": role.team_id,
                "person_id": role.person_id,
                "reports_to": role.reports_to,
            })),
            None,
        );

        Ok(role)
    }

    #[graphql(
        name = "updateRole",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_role(
        &self,
        context: &Context<'_>,
        role_data: RoleData,
    ) -> Result<Role> {

        let mut role = Role::get_by_id(&role_data.id)?;
        authz::require_manage_team(context, &role.team_id)?;

        if let Some(id) = role_data.active {
            role.active = id;
        };

        if let Some(s) = role_data.title_en {
            if !s.trim().is_empty() { role.title_en = s.trim().to_string(); }
        };

        if let Some(s) = role_data.title_fr {
            if !s.trim().is_empty() { role.title_fr = s.trim().to_string(); }
        };

        if let Some(s) = role_data.start_datestamp {
            role.start_datestamp = s;
        };

        if let Some(s) = role_data.end_date {
            role.end_date = Some(s);
        };

        if let Some(effort) = role_data.effort {
            if !effort.is_finite() || effort < 0.0 {
                return Err(Error::new("Effort cannot be negative"));
            }
            role.effort = effort;
        };

        // A classification change replaces the whole classification, must
        // still suit the incumbent, and must keep every reporting line
        // pointing at a more senior position.
        if let Some(classification) = &role_data.classification {
            classification.validate()?;
            if let Some(person_id) = role.person_id {
                require_classification_for(&person_id, classification)?;
            }
            role.military_occupation = classification.military_occupation;
            role.rank = classification.rank;
            role.occupational_group = classification.occupational_group;
            role.occupational_level = classification.occupational_level;
            role.check_reporting_lines()?;
        };

        // Persist the change. Without this the mutation silently returned the
        // in-memory edit while the database row was left untouched.
        let saved = role.update()?;
        let role = if role_data.classification.is_some() { role.save_classification()? } else { saved };

        AuditEvent::log(
            context,
            "role.update",
            "role",
            Some(role.id),
            format!("Updated role “{}”", role.title_en),
            Some(json!({
                "active": role.active,
                "start_datestamp": role.start_datestamp,
                "end_date": role.end_date,
                "effort": role.effort,
                "rank": role.rank,
                "occupational_group": role.occupational_group,
                "occupational_level": role.occupational_level,
            })),
            None,
        );

        Ok(role)
    }

    /// Assign a person to a vacant role. Errors if the role is already occupied.
    #[graphql(
        name = "assignPersonToRole",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn assign_person_to_role(
        &self,
        context: &Context<'_>,
        person_id: Uuid,
        role_id: Uuid,
    ) -> Result<Role> {
        authz::require_manage_role(context, &role_id)?;
        let role = Role::assign_person(&role_id, &person_id)?;

        AuditEvent::log(
            context,
            "role.assign",
            "role",
            Some(role.id),
            format!("Assigned person to role “{}”", role.title_en),
            Some(json!({ "person_id": person_id })),
            None,
        );

        Ok(role)
    }

    /// Remove the person from a role, leaving it vacant.
    #[graphql(
        name = "vacateRole",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn vacate_role(
        &self,
        context: &Context<'_>,
        role_id: Uuid,
    ) -> Result<Role> {
        authz::require_manage_role(context, &role_id)?;
        let role = Role::vacate(&role_id)?;

        AuditEvent::log(
            context,
            "role.vacate",
            "role",
            Some(role.id),
            format!("Vacated role “{}”", role.title_en),
            None,
            None,
        );

        Ok(role)
    }

    /// Set (or clear) the position a role reports to. Pass `reports_to_role_id`
    /// = null to clear the explicit edge and fall back to the team owner.
    /// Rejects self-references and cycles. The caller must manage the role's
    /// team.
    #[graphql(
        name = "setRoleReportsTo",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn set_role_reports_to(
        &self,
        context: &Context<'_>,
        role_id: Uuid,
        reports_to_role_id: Option<Uuid>,
    ) -> Result<Role> {
        authz::require_manage_role(context, &role_id)?;
        let role = Role::set_reports_to(&role_id, reports_to_role_id)?;

        AuditEvent::log(
            context,
            "role.reports_to.set",
            "role",
            Some(role.id),
            "Set reporting line".to_string(),
            Some(json!({ "reports_to": reports_to_role_id })),
            None,
        );

        Ok(role)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
/// InputObject for Role with Option fields - only include the ones you want to update.
/// Corrections to the position (titles, effort, classification) belong here;
/// a different job is a new role, so history and progression stay visible.
pub struct RoleData {
    pub id: Uuid,
    pub active: Option<bool>,

    /// Correct the position's title (e.g. a typo). The position itself is
    /// durable — retitle, don't recreate, when the role is the same job.
    pub title_en: Option<String>,
    pub title_fr: Option<String>,

    pub start_datestamp: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,

    /// Expected workload for the position; full time is around 10.
    pub effort: Option<f64>,

    /// Replace the role's classification as a whole (see `RoleClassification`).
    pub classification: Option<RoleClassification>,
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
/// A role's classification, replaced as a whole when given: a military
/// occupation and rank, or a civilian occupational group and level, or
/// neither for an unclassified role. Giving one stream clears the other.
pub struct RoleClassification {
    pub military_occupation: Option<MilitaryOccupation>,
    pub rank: Option<Rank>,
    pub occupational_group: Option<OccupationalGroup>,
    pub occupational_level: Option<i32>,
}

impl RoleClassification {
    fn validate(&self) -> Result<()> {
        let military = self.military_occupation.is_some() || self.rank.is_some();
        let civilian = self.occupational_group.is_some() || self.occupational_level.is_some();
        if military && civilian {
            return Err(Error::new("A role is classified as military or civilian, not both"));
        }
        Ok(())
    }
}

/// A person's personnel type requires a matching classification on the role
/// they hold: occupation and rank for military personnel, group and level for
/// civilians. Contractors, students and others have no required HR fields.
fn require_classification_for(person_id: &Uuid, c: &RoleClassification) -> Result<()> {
    match Person::get_by_id(person_id)?.personnel_type {
        PersonnelType::Military if c.military_occupation.is_none() || c.rank.is_none() => Err(Error::new(
            "Roles assigned to military personnel require a military_occupation and rank")),
        PersonnelType::Civilian if c.occupational_group.is_none() || c.occupational_level.is_none() => Err(Error::new(
            "Roles assigned to civilian personnel require an occupational_group and occupational_level")),
        _ => Ok(()),
    }
}