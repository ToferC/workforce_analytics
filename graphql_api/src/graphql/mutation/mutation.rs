use async_graphql::*;

// use rdkafka::producer::FutureProducer;
// use crate::kafka::send_message;

use crate::graphql::mutation::{
    CapabilityMutation, PersonMutation, RoleMutation, RoleOfferMutation, UserMutation, SkillMutation,
    OrganizationMutation, OrgTierMutation, OrgOwnershipMutation,
    TeamMutation, TeamOwnershipMutation, AffiliationMutation,
    PublicationMutation, PublicationContributorMutation,
    WorkMutation, TaskMutation, ProductMutation, RequirementMutation,
    ValidationMutation, LanguageMutation, SelfServiceMutation,
};

#[derive(MergedObject, Default)]
pub struct Mutation(
    UserMutation,
    PersonMutation,
    RoleMutation,
    RoleOfferMutation,
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
    ProductMutation,
    RequirementMutation,
    ValidationMutation,
    LanguageMutation,
    SelfServiceMutation,
);