//! Work groups need several models acting as intermediary between the interface and the database

use crate::model::security::Role;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Command to register a new entity
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Insertable)]
#[serde(rename_all = "camelCase")]
#[schema(as = WorkgroupRegisterCommand)]
#[diesel(table_name = crate::schema::workgroups)]
pub struct RegisterCommand {
    #[schema(example = "Foo Group")]
    pub name: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Queryable, Identifiable, Selectable)]
#[serde(rename_all = "camelCase")]
#[schema(as = WorkgroupEntity)]
#[diesel(table_name = crate::schema::workgroups)]
pub struct Entity {
    #[serde(default)]
    pub id: i32,

    #[schema(example = "Foo Group")]
    pub name: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Queryable, Selectable)]
#[serde(rename_all = "camelCase")]
#[schema(as = WorkgroupRoleAssociation)]
#[diesel(table_name = crate::schema::workgroup_role_associations)]
pub struct RoleAssociation {
    #[schema(example = 1)]
    pub workgroup_id: i32,

    #[schema(example = Role::OrchestraCommittee)]
    pub system_role: Role,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Queryable, Selectable)]
#[serde(rename_all = "camelCase")]
#[schema(as = WorkgroupMemberRelationship)]
#[diesel(table_name = crate::schema::workgroup_member_relationships)]
pub struct MemberRelationship {
    #[schema(example = 1)]
    pub workgroup_id: i32,

    #[schema(example = 1)]
    pub member_id: i32,
}
