use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use serde_json::json;

use crate::models::{Role, RoleAssignment, NewRole, Person, PersonnelType, AuditEvent};
use crate::common_utils::{UserRole,
    is_operator, RoleGuard};
use crate::graphql::authz;
use crate::schema::roles;
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
            let person = Person::get_by_id(&person_id)?;

            match person.personnel_type {
                PersonnelType::Military => {
                    if role_data.military_occupation.is_none() || role_data.rank.is_none() {
                        return Err(Error::new(
                            "Roles assigned to military personnel require a military_occupation and rank"));
                    }
                },
                PersonnelType::Civilian => {
                    if role_data.occupational_group.is_none() || role_data.occupational_level.is_none() {
                        return Err(Error::new(
                            "Roles assigned to civilian personnel require an occupational_group and occupational_level"));
                    }
                },
                // Contractors, students and others have no required HR fields
                _ => (),
            }
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

        if let Some(s) = role_data.start_datestamp {
            role.start_datestamp = s;
        };

        if let Some(s) = role_data.end_date {
            role.end_date = Some(s);
        };

        // Persist the change. Without this the mutation silently returned the
        // in-memory edit while the database row was left untouched.
        let role = role.update()?;

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

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Insertable, AsChangeset, InputObject)]
#[graphql(complex)]
#[diesel(table_name = roles)]
/// InputObject for Role with Option fields - only include the ones you want to update
/// Note the only changable fields are active, start and end.
/// It's recommended to create new roles for people vs. edit existing ones to show history
/// and progression
pub struct RoleData {
    pub id: Uuid,
    pub active: Option<bool>,

    pub start_datestamp: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
}