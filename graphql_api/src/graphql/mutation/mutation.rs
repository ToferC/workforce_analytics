use async_graphql::*;

// use rdkafka::producer::FutureProducer;
// use crate::kafka::send_message;

use crate::graphql::mutation::{
    CapabilityMutation, PersonMutation, RoleMutation, UserMutation, SkillMutation,
    OrganizationMutation, OrgTierMutation, OrgOwnershipMutation,
    TeamMutation, TeamOwnershipMutation, AffiliationMutation,
    PublicationMutation, PublicationContributorMutation,
    WorkMutation, TaskMutation, RequirementMutation,
    ValidationMutation, LanguageMutation,
};

#[derive(MergedObject, Default)]
pub struct Mutation(
    UserMutation,
    PersonMutation,
    RoleMutation,
    CapabilityMutation,
    SkillMutation,
    OrganizationMutation,
    OrgTierMutation,
    OrgOwnershipMutation,
    TeamMutation,
    TeamOwnershipMutation,
    AffiliationMutation,
    PublicationMutation,
    PublicationContributorMutation,
    WorkMutation,
    TaskMutation,
    RequirementMutation,
    ValidationMutation,
    LanguageMutation,
);