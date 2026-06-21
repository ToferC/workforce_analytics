use std::str::FromStr;

use async_graphql::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::models::{InsertableUser, LoginQuery,
    User, UserData, create_token, decode_token,
    verify_password, UserUpdate, hash_password, Person,
    STATUS_ACTIVE, STATUS_DISABLED, STATUS_PROVISIONED};
use crate::common_utils::{UserRole,
    is_admin, is_operator, RoleGuard};
// use rdkafka::producer::FutureProducer;
// use crate::kafka::send_message;

#[derive(Default)]
pub struct UserMutation;

#[derive(Debug, Serialize, Deserialize, SimpleObject)]
pub struct UserResponse {
    id: Uuid,
    bearer: String,
    role: String,
    email: String,
    expires_at: NaiveDateTime,
}

/// Result of issuing an activation invite. The caller surfaces the token as part
/// of an activation link (e.g. /activate?token=…).
#[derive(Debug, Serialize, Deserialize, SimpleObject)]
pub struct InviteResult {
    pub user_id: Uuid,
    pub activation_token: String,
    pub expires_at: NaiveDateTime,
}

// Mutation Example

#[Object]
impl UserMutation {
    /*
    #[graphql(
        name = "PILQuery", 
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    /// Receives a Vec<TravelData> containing details from a group of travllers
    /// and returns a Vec<TravelResponse> containing public health direction for the BSO
    /// relating to entry to Canada for public health reasons and referrals to mandatory
    /// random testing. Also includes IDs for Person, Trip, QuarantinePlan
    /// for further mutations.
    pub async fn travel_data_response(
        &self,
        context: &Context<'_>,
        data: Vec<TravelData>,
    ) -> FieldResult<Vec<PILResponse>> {

        let cbsa_id = context.data_opt::<Uuid>().expect("Unable to parse CBSA ID");

        let mut responses_to_cbsa: Vec<PILResponse> = Vec::new();

        let travel_group_id = Uuid::new_v4();

        for traveller in data {
            let response = traveller.process(&context, travel_group_id, *cbsa_id)
                .await?
                .into();
                
            responses_to_cbsa.push(response);

            /* 
            // Create Kafka producer and send message for subscription service
            let producer = context
                .data::<FutureProducer>()
                .expect("Can't get Kafka producer");
            */

            // Sent ArriveCan messages to Kafka
            let arrivecan_message = serde_json::to_string(&traveller)
                .expect("Can't serialize ArriveCan PIL message");

            /* 
            // Remove subscription until we set up Kafka service
            println!("Sending ArriveCan PIL Message to Subscription");
            send_message(producer, "arrivecan_pil", arrivecan_message, "CBSA".to_string()).await;
            */
        };        
        
        Ok(responses_to_cbsa)
    }
*/
    #[graphql(
        name = "createUser",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn create_user(
        &self,
        _context: &Context<'_>,
        user_data: UserData,
    ) -> FieldResult<User> {
        let new_user = InsertableUser::from(user_data);

        let created_user = User::create(new_user);

        created_user
    }

    #[graphql(
        name = "updateUser",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn update_user(
        &self,
        _context: &Context<'_>,
        user_data: UserUpdate,
    ) -> FieldResult<User> {
        let mut target_user = User::get_by_id(&user_data.id)?;

        if let Some(s) = user_data.name {
            target_user.name = s;
        };

        if let Some(s) = user_data.email {
            target_user.email = s;
        };

        if let Some(s) = user_data.password {
            target_user.hash = hash_password(&s)?.to_string();
        };

        if let Some(s) = user_data.role {
            target_user.role = s;
        };

        let updated_user = target_user.update();

        updated_user
    }

    /// Grant a provisioned account a path to access: issue an activation token
    /// and move the user to INVITED. Returns the token + expiry so the operator
    /// can share an activation link (no email is sent in v1).
    #[graphql(
        name = "inviteUser",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn invite_user(
        &self,
        _context: &Context<'_>,
        user_id: Uuid,
    ) -> Result<InviteResult> {
        let user = User::get_by_id(&user_id)?;
        if user.email.trim().is_empty() {
            return Err(Error::new("Cannot invite a user without an email address."));
        }
        let (token, expires_at) = User::invite(&user_id)?;
        Ok(InviteResult { user_id, activation_token: token, expires_at })
    }

    /// Redeem an activation token by setting a password. Public: this is how an
    /// invited person gains access. Moves the account to ACTIVE.
    #[graphql(name = "activateAccount")]
    pub async fn activate_account(
        &self,
        _context: &Context<'_>,
        token: String,
        password: String,
    ) -> Result<bool> {
        if password.trim().len() < 8 {
            return Err(Error::new("Password must be at least 8 characters."));
        }
        User::activate(token.trim(), &password)?;
        Ok(true)
    }

    /// Revoke a user's access (record retained). Admin only.
    #[graphql(
        name = "disableUser",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn disable_user(
        &self,
        _context: &Context<'_>,
        user_id: Uuid,
    ) -> Result<User> {
        User::set_status(&user_id, STATUS_DISABLED)
    }

    /// Re-enable a disabled user. Restores ACTIVE if they have a password,
    /// otherwise PROVISIONED (they still need to activate). Admin only.
    #[graphql(
        name = "enableUser",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn enable_user(
        &self,
        _context: &Context<'_>,
        user_id: Uuid,
    ) -> Result<User> {
        let user = User::get_by_id(&user_id)?;
        let status = if user.hash.is_empty() { STATUS_PROVISIONED } else { STATUS_ACTIVE };
        User::set_status(&user_id, status)
    }

    /// Grant access to a person's account by person id, for operators who manage
    /// people but cannot read user records directly (userByEmail is admin-only).
    /// Resolves the person's account and issues an activation invite. Refuses if
    /// the account is already ACTIVE, so it is safe to call without first
    /// inspecting account status.
    #[graphql(
        name = "invitePerson",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn invite_person(
        &self,
        _context: &Context<'_>,
        person_id: Uuid,
    ) -> Result<InviteResult> {
        let person = Person::get_by_id(&person_id)?;
        let user = User::get_by_id(&person.user_id)?;

        if user.email.trim().is_empty() {
            return Err(Error::new("Cannot invite a user without an email address."));
        }
        if user.status == STATUS_ACTIVE {
            return Err(Error::new("This person's account is already active."));
        }

        let (token, expires_at) = User::invite(&person.user_id)?;
        Ok(InviteResult { user_id: person.user_id, activation_token: token, expires_at })
    }

    pub async fn sign_in(
        &self,
        _context: &Context<'_>,
        input: LoginQuery,
    ) -> Result<UserResponse, Error> {
        let maybe_user = User::get_by_email(&input.email).ok();

        if let Some(user) = maybe_user {

            if let Ok(matching) = verify_password(user.hash.to_string(), &input.password) {
                if matching {
                    // Only ACTIVE accounts may sign in. PROVISIONED/INVITED
                    // accounts have no access until activated; DISABLED are
                    // revoked.
                    if user.status != STATUS_ACTIVE {
                        return Err(Error::new(
                            "This account is not active. Ask an administrator to grant access, then activate your invitation.",
                        ));
                    }

                    let role = UserRole::from_str(user.role.as_str())
                        .expect("Cannot convert &str to UserRole");

                    // Return the token which would be accepted by the Epicenter 
                    // app and used to authenticate actions
                    let (token, expiry) = create_token(user.id.to_string(), role)?;

                    let res = UserResponse {
                        id: user.id,
                        email: user.email.to_owned(),
                        bearer: token.to_owned(),
                        role: user.role,
                        expires_at: expiry.naive_local(),
                    };


                    println!("JWT: {}\nData{:?}", &token, decode_token(&token));

                    return Ok(res);
                }
            }
        }

        Err(Error::new("Can't authenticate a user"))
    }
}