use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberRoleAssociation {
    #[schema(example = 1)]
    member_id: i32,
    #[schema(example = 1)]
    system_role: i32,
}
