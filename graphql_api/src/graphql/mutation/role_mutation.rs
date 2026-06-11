use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Role, NewRole, Person, PersonnelType};
use crate::common_utils::{UserRole,
    is_operator, RoleGuard};
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
        _context: &Context<'_>,
        role_data: NewRole,
    ) -> Result<Role> {

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

        let role = Role::create(&role_data)?;

        Ok(role)
    }

    #[graphql(
        name = "updateRole", 
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_role(
        &self,
        _context: &Context<'_>,
        role_data: RoleData,
    ) -> Result<Role> {
        
        let mut role = Role::get_by_id(&role_data.id)?;

        if let Some(id) = role_data.active {
            role.active = id;
        };

        if let Some(s) = role_data.start_datestamp {
            role.start_datestamp = s;
        };

        if let Some(s) = role_data.end_date {
            role.end_date = Some(s);
        };

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