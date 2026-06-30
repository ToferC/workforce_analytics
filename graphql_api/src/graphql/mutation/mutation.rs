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

// async-graphql expands a MergedObject into a single `resolve_field` async
// state machine whose type-layout depth grows with the member count; a flat
// 21-member Mutation overflows the compiler's default recursion limit (128)
// when its layout is computed during codegen. Grouping the members into nested
// MergedObjects keeps each generated function small. The GraphQL schema is
// unchanged — MergedObject flattens, so every mutation still appears at the top
// level of `Mutation`.
#[derive(MergedObject, Default)]
pub struct Mutation(
    PeopleMutations,
    OrgMutations,
    WorkMutations,
);

#[derive(MergedObject, Default)]
pub struct PeopleMutations(
    UserMutation,
    PersonMutation,
    RoleMutation,
    RoleOfferMutation,
    CapabilityMutation,
    SkillMutation,
    ValidationMutation,
);

#[derive(MergedObject, Default)]
pub struct OrgMutations(
    OrganizationMutation,
    OrgTierMutation,
    OrgOwnershipMutation,
    TeamMutation,
    TeamOwnershipMutation,
    AffiliationMutation,
    LanguageMutation,
);

#[derive(MergedObject, Default)]
pub struct WorkMutations(
    PublicationMutation,
    PublicationContributorMutation,
    WorkMutation,
    TaskMutation,
    ProductMutation,
    RequirementMutation,
    SelfServiceMutation,
);