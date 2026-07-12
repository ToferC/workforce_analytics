// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "capability_level"))]
    pub struct CapabilityLevel;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "contract_status"))]
    pub struct ContractStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "language_level"))]
    pub struct LanguageLevel;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "language_name"))]
    pub struct LanguageName;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "military_occupation"))]
    pub struct MilitaryOccupation;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "occupational_group"))]
    pub struct OccupationalGroup;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "personnel_type"))]
    pub struct PersonnelType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "priority"))]
    pub struct Priority;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "publication_status"))]
    pub struct PublicationStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "rank"))]
    pub struct Rank;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "role_offer_status"))]
    pub struct RoleOfferStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "skill_domain"))]
    pub struct SkillDomain;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "work_status"))]
    pub struct WorkStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "work_update_kind"))]
    pub struct WorkUpdateKind;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "approval_status"))]
    pub struct ApprovalStatus;
}

diesel::table! {
    affiliations (id) {
        id -> Uuid,
        person_id -> Uuid,
        organization_id -> Uuid,
        home_org_id -> Uuid,
        #[max_length = 256]
        affiliation_role -> Varchar,
        start_datestamp -> Timestamp,
        end_date -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    audit_events (id) {
        id -> Uuid,
        occurred_at -> Timestamp,
        actor_user_id -> Nullable<Uuid>,
        actor_role_id -> Nullable<Uuid>,
        #[max_length = 64]
        action -> Varchar,
        #[max_length = 48]
        entity_type -> Varchar,
        entity_id -> Nullable<Uuid>,
        summary -> Nullable<Text>,
        payload -> Nullable<Jsonb>,
        correlation_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RoleOfferStatus;

    role_offers (id) {
        id -> Uuid,
        role_id -> Uuid,
        person_id -> Uuid,
        offered_by_role_id -> Uuid,
        from_role_id -> Nullable<Uuid>,
        approver_role_id -> Nullable<Uuid>,
        status -> RoleOfferStatus,
        message -> Nullable<Text>,
        decision_note -> Nullable<Text>,
        decided_by_role_id -> Nullable<Uuid>,
        decided_at -> Nullable<Timestamp>,
        expires_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SkillDomain;
    use super::sql_types::CapabilityLevel;

    capabilities (id) {
        id -> Uuid,
        #[max_length = 256]
        name_en -> Varchar,
        #[max_length = 256]
        name_fr -> Varchar,
        domain -> SkillDomain,
        person_id -> Uuid,
        skill_id -> Uuid,
        organization_id -> Uuid,
        self_identified_level -> CapabilityLevel,
        validated_level -> Nullable<CapabilityLevel>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        retired_at -> Nullable<Timestamp>,
        validated_by_id -> Nullable<Uuid>,
        validated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LanguageName;
    use super::sql_types::LanguageLevel;

    language_datas (id) {
        id -> Uuid,
        person_id -> Uuid,
        language_name -> LanguageName,
        reading -> Nullable<LanguageLevel>,
        writing -> Nullable<LanguageLevel>,
        speaking -> Nullable<LanguageLevel>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    org_tier_ownerships (id) {
        id -> Uuid,
        owner_role_id -> Uuid,
        org_tier_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        retired_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SkillDomain;

    org_tiers (id) {
        id -> Uuid,
        organization_id -> Uuid,
        tier_level -> Int4,
        #[max_length = 256]
        name_en -> Varchar,
        #[max_length = 256]
        name_fr -> Varchar,
        primary_domain -> SkillDomain,
        parent_tier -> Nullable<Uuid>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        retired_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    organizations (id) {
        id -> Uuid,
        #[max_length = 256]
        name_en -> Varchar,
        #[max_length = 256]
        name_fr -> Varchar,
        #[max_length = 16]
        acronym_en -> Varchar,
        #[max_length = 16]
        acronym_fr -> Varchar,
        #[max_length = 32]
        org_type -> Varchar,
        #[max_length = 256]
        url -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        retired_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PersonnelType;

    persons (id) {
        id -> Uuid,
        user_id -> Uuid,
        family_name -> Varchar,
        given_name -> Varchar,
        #[max_length = 128]
        email -> Varchar,
        #[max_length = 32]
        phone -> Varchar,
        #[max_length = 256]
        work_address -> Varchar,
        #[max_length = 128]
        city -> Varchar,
        #[max_length = 128]
        province -> Varchar,
        #[max_length = 16]
        postal_code -> Varchar,
        #[max_length = 128]
        country -> Varchar,
        organization_id -> Uuid,
        peoplesoft_id -> Varchar,
        orcid_id -> Varchar,
        personnel_type -> PersonnelType,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        retired_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SkillDomain;
    use super::sql_types::WorkStatus;
    use super::sql_types::Priority;

    products (id) {
        id -> Uuid,
        organization_id -> Uuid,
        product_owner_role_id -> Uuid,
        #[max_length = 256]
        name_en -> Varchar,
        #[max_length = 256]
        name_fr -> Varchar,
        description_en -> Text,
        description_fr -> Text,
        primary_domain -> SkillDomain,
        #[max_length = 256]
        url -> Nullable<Varchar>,
        product_status -> WorkStatus,
        priority -> Priority,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        retired_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    publication_contributors (id) {
        id -> Uuid,
        publication_id -> Uuid,
        contributor_id -> Uuid,
        #[max_length = 256]
        contributor_role -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    record_flags (id) {
        id -> Uuid,
        person_id -> Uuid,
        message -> Varchar,
        created_at -> Timestamp,
        resolved_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PublicationStatus;

    publications (id) {
        id -> Uuid,
        publishing_organization_id -> Uuid,
        lead_author_id -> Uuid,
        #[max_length = 256]
        title -> Varchar,
        #[max_length = 256]
        subject_text -> Varchar,
        publication_status -> PublicationStatus,
        #[max_length = 256]
        url_string -> Nullable<Varchar>,
        #[max_length = 256]
        publishing_id -> Nullable<Varchar>,
        submitted_date -> Nullable<Timestamp>,
        published_datestamp -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SkillDomain;
    use super::sql_types::CapabilityLevel;

    requirements (id) {
        id -> Uuid,
        #[max_length = 256]
        name_en -> Varchar,
        #[max_length = 256]
        name_fr -> Varchar,
        domain -> SkillDomain,
        role_id -> Uuid,
        skill_id -> Uuid,
        required_level -> CapabilityLevel,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        retired_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MilitaryOccupation;
    use super::sql_types::Rank;
    use super::sql_types::OccupationalGroup;

    roles (id) {
        id -> Uuid,
        person_id -> Nullable<Uuid>,
        team_id -> Uuid,
        #[max_length = 256]
        title_en -> Varchar,
        #[max_length = 256]
        title_fr -> Varchar,
        effort -> Float8,
        active -> Bool,
        military_occupation -> Nullable<MilitaryOccupation>,
        rank -> Nullable<Rank>,
        occupational_group -> Nullable<OccupationalGroup>,
        occupational_level -> Nullable<Int4>,
        start_datestamp -> Timestamp,
        end_date -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        reports_to -> Nullable<Uuid>,
        annual_salary_cents -> Nullable<Int8>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Rank;
    use super::sql_types::OccupationalGroup;

    pay_rates (id) {
        id -> Uuid,
        occupational_group -> Nullable<OccupationalGroup>,
        occupational_level -> Nullable<Int4>,
        rank -> Nullable<Rank>,
        annual_rate_cents -> Int8,
        effective_date -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ContractStatus;

    contracts (id) {
        id -> Uuid,
        task_id -> Uuid,
        #[max_length = 64]
        reference_number -> Varchar,
        #[max_length = 256]
        vendor -> Varchar,
        description -> Text,
        start_date -> Timestamp,
        end_date -> Timestamp,
        total_value_cents -> Int8,
        status -> ContractStatus,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    budget_allocations (id) {
        id -> Uuid,
        org_tier_id -> Uuid,
        fiscal_year -> Int4,
        amount_cents -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    role_assignments (id) {
        id -> Uuid,
        role_id -> Uuid,
        person_id -> Uuid,
        start_date -> Timestamp,
        end_date -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SkillDomain;

    skills (id) {
        id -> Uuid,
        #[max_length = 256]
        name_en -> Varchar,
        #[max_length = 256]
        name_fr -> Varchar,
        description_en -> Text,
        description_fr -> Text,
        domain -> SkillDomain,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        retired_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SkillDomain;
    use super::sql_types::WorkStatus;
    use super::sql_types::Priority;
    use super::sql_types::ApprovalStatus;

    tasks (id) {
        id -> Uuid,
        created_by_role_id -> Uuid,
        #[max_length = 144]
        title -> Varchar,
        domain -> SkillDomain,
        #[max_length = 1256]
        intended_outcome -> Varchar,
        #[max_length = 1256]
        final_outcome -> Nullable<Varchar>,
        approval_tier -> Int4,
        #[max_length = 256]
        url -> Varchar,
        start_datestamp -> Timestamp,
        target_completion_date -> Timestamp,
        task_status -> WorkStatus,
        priority -> Priority,
        completed_date -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        product_id -> Nullable<Uuid>,
        approval_status -> ApprovalStatus,
        approved_by_user_id -> Nullable<Uuid>,
        approved_at -> Nullable<Timestamp>,
        rejection_reason -> Nullable<Text>,
    }
}

diesel::table! {
    team_ownerships (id) {
        id -> Uuid,
        owner_role_id -> Uuid,
        team_id -> Uuid,
        start_datestamp -> Timestamp,
        end_date -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SkillDomain;

    teams (id) {
        id -> Uuid,
        organization_id -> Uuid,
        org_tier_id -> Uuid,
        primary_domain -> SkillDomain,
        #[max_length = 256]
        name_en -> Varchar,
        #[max_length = 256]
        name_fr -> Varchar,
        description_en -> Text,
        description_fr -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        retired_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        hash -> Varchar,
        #[max_length = 128]
        email -> Varchar,
        #[max_length = 64]
        role -> Varchar,
        #[max_length = 256]
        name -> Varchar,
        #[max_length = 64]
        access_level -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        #[max_length = 256]
        access_key -> Varchar,
        approved_by_user_uid -> Nullable<Uuid>,
        #[max_length = 16]
        account_type -> Varchar,
        #[max_length = 16]
        status -> Varchar,
        #[max_length = 128]
        activation_token -> Nullable<Varchar>,
        activation_expires_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    valid_roles (role) {
        #[max_length = 64]
        role -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CapabilityLevel;

    validations (id) {
        id -> Uuid,
        validator_id -> Uuid,
        capability_id -> Uuid,
        validated_level -> CapabilityLevel,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    work_dependencies (id) {
        id -> Uuid,
        work_id -> Uuid,
        depends_on_work_id -> Uuid,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::WorkUpdateKind;

    work_updates (id) {
        id -> Uuid,
        work_id -> Uuid,
        author_user_id -> Nullable<Uuid>,
        kind -> WorkUpdateKind,
        body -> Text,
        created_at -> Timestamp,
        flag_resolved_at -> Nullable<Timestamp>,
        resolved_by_user_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::WorkStatus;

    work_status_history (id) {
        id -> Uuid,
        work_id -> Uuid,
        from_status -> Nullable<WorkStatus>,
        to_status -> WorkStatus,
        changed_at -> Timestamp,
        changed_by_user_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SkillDomain;
    use super::sql_types::CapabilityLevel;
    use super::sql_types::WorkStatus;
    use super::sql_types::Priority;

    works (id) {
        id -> Uuid,
        task_id -> Uuid,
        role_id -> Nullable<Uuid>,
        #[max_length = 256]
        work_description -> Varchar,
        #[max_length = 256]
        url -> Nullable<Varchar>,
        domain -> SkillDomain,
        capability_level -> CapabilityLevel,
        effort -> Int4,
        work_status -> WorkStatus,
        priority -> Priority,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        skill_id -> Uuid,
        due_date -> Nullable<Timestamp>,
        started_at -> Nullable<Timestamp>,
        completed_at -> Nullable<Timestamp>,
        #[max_length = 1024]
        blocked_reason -> Nullable<Varchar>,
        blocked_since -> Nullable<Timestamp>,
        blocked_on_role_id -> Nullable<Uuid>,
    }
}

diesel::joinable!(affiliations -> persons (person_id));
diesel::joinable!(capabilities -> organizations (organization_id));
diesel::joinable!(capabilities -> persons (person_id));
diesel::joinable!(capabilities -> skills (skill_id));
diesel::joinable!(language_datas -> persons (person_id));
diesel::joinable!(org_tier_ownerships -> org_tiers (org_tier_id));
diesel::joinable!(org_tier_ownerships -> roles (owner_role_id));
diesel::joinable!(org_tiers -> organizations (organization_id));
diesel::joinable!(persons -> organizations (organization_id));
diesel::joinable!(persons -> users (user_id));
diesel::joinable!(record_flags -> persons (person_id));diesel::joinable!(products -> organizations (organization_id));
diesel::joinable!(products -> roles (product_owner_role_id));
diesel::joinable!(publication_contributors -> persons (contributor_id));
diesel::joinable!(publication_contributors -> publications (publication_id));
diesel::joinable!(publications -> organizations (publishing_organization_id));
diesel::joinable!(publications -> persons (lead_author_id));
diesel::joinable!(requirements -> roles (role_id));
diesel::joinable!(requirements -> skills (skill_id));
diesel::joinable!(role_assignments -> persons (person_id));
diesel::joinable!(role_assignments -> roles (role_id));
diesel::joinable!(roles -> persons (person_id));
diesel::joinable!(roles -> teams (team_id));
diesel::joinable!(tasks -> products (product_id));
diesel::joinable!(tasks -> roles (created_by_role_id));
diesel::joinable!(team_ownerships -> roles (owner_role_id));
diesel::joinable!(team_ownerships -> teams (team_id));
diesel::joinable!(teams -> org_tiers (org_tier_id));
diesel::joinable!(teams -> organizations (organization_id));
diesel::joinable!(users -> valid_roles (role));
diesel::joinable!(validations -> capabilities (capability_id));
diesel::joinable!(validations -> users (validator_id));
diesel::joinable!(works -> roles (role_id));
diesel::joinable!(works -> skills (skill_id));
diesel::joinable!(works -> tasks (task_id));
diesel::joinable!(contracts -> tasks (task_id));
diesel::joinable!(budget_allocations -> org_tiers (org_tier_id));

diesel::allow_tables_to_appear_in_same_query!(
    affiliations,
    audit_events,
    budget_allocations,
    capabilities,
    contracts,
    language_datas,
    org_tier_ownerships,
    org_tiers,
    organizations,
    pay_rates,
    persons,
    products,
    publication_contributors,
    publications,
    record_flags,
    requirements,
    role_assignments,
    role_offers,
    roles,
    skills,
    tasks,
    team_ownerships,
    teams,
    users,
    valid_roles,
    validations,
    work_dependencies,
    work_status_history,
    work_updates,
    works,
);
