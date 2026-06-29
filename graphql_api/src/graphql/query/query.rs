use async_graphql::*;

use crate::graphql::query::{CapabilityQuery, PersonQuery, TeamQuery, OrganizationQuery, UserQuery, RoleQuery, AnalyticsQuery, SelfServiceQuery, AuditQuery, RoleOfferQuery};

use super::{ProductQuery, PublicationQuery, TaskQuery, WorkQuery};

#[derive(Default, MergedObject)]
pub struct Query(
    CapabilityQuery,
    PersonQuery,
    TeamQuery,
    OrganizationQuery,
    UserQuery,
    RoleQuery,
    PublicationQuery,
    TaskQuery,
    WorkQuery,
    ProductQuery,
    AnalyticsQuery,
    SelfServiceQuery,
    AuditQuery,
    RoleOfferQuery,
);