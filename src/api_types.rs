use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ZoneConnection {
    pub name: String,
    pub key_name: String,
    pub key: String,
    pub primary_server: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ACLRule {
    pub access_level: String,
    pub description: String,
    pub user_id: String,
    pub group_id: String,
    pub record_mask: String,
    pub record_types: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ZoneACL {
    pub rules: Vec<ACLRule>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    pub name: String,
    pub email: String,
    pub status: String,
    pub created: String,
    pub id: String,
    pub admin_group_id: String,
    pub latest_sync: String,
    pub updated: String,
    pub connection: ZoneConnection,
    pub transfer_connection: ZoneConnection,
    pub acl: ZoneACL,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ZoneResponse {
    pub zones: Vec<Zone>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ZoneHistory {
    pub zone_id: String,
    pub zone_chanes: Vec<ZoneChange>,
    pub record_set_changes: Vec<RecordSetChange>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ZoneChange {
    pub zone: Zone,
    pub user_id: String,
    pub change_type: String,
    pub status: String,
    pub created: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordSetChange {
    pub zone: Zone,
    pub record_set: RecordSet,
    pub user_id: String,
    pub change_type: String,
    pub status: String,
    pub created: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordSet {
    pub id: String,
    pub zone_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub status: String,
    pub created: String,
    pub updated: String,
    pub ttl: i32,
    pub account: String,
    pub records: Vec<Record>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordSetUpdateResponse {
    pub zone: Zone,
    pub record_set: RecordSet,
    pub id: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Record {
    pub address: String,
    pub cname: String,
    pub preference: i32,
    pub exchange: String,
    pub nsdname: String,
    pub ptrdname: String,
    pub mname: String,
    pub rname: String,
    pub serial: i32,
    pub refresh: i32,
    pub retry: i32,
    pub expire: i32,
    pub minimum: i32,
    pub text: String,
    pub priority: i32,
    pub weight: i32,
    pub port: i32,
    pub target: String,
    pub algorithm: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub fingerprint: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordSetResponse {
    pub record_set: RecordSet,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordSetsResponse {
    #[serde(default)]
    pub next_id: String,
    pub record_sets: Vec<RecordSet>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct User {
    pub id: String,
    pub user_name: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub created: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Groups {
    pub groups: Vec<Group>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub created: String,
    pub members: Vec<User>,
    pub admins: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GroupAdmins {
    pub admins: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GroupMembers {
    pub members: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GroupChange {
    pub user_id: String,
    pub created: String,
    pub change_type: String,
    pub new_group: Group,
    pub old_group: Group,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GroupChanges {
    pub changes: Vec<GroupChange>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BatchRecordChanges {
    pub batch_changes: Vec<RecordChange>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordChange {
    pub id: String,
    pub status: String,
    pub change_type: String,
    pub record_name: String,
    pub ttl: i32,
    #[serde(rename = "type")]
    pub typ: String,
    pub zone_name: String,
    pub input_name: String,
    pub zone_id: String,
    pub total_changes: i32,
    pub user_name: String,
    pub comments: String,
    pub user_id: String,
    pub created_timestamp: String,
    pub data: RecordData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BatchRecordChangeUpdateResponse {
    pub comments: String,
    pub changes: Vec<RecordChange>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordData {
    pub address: String,
    pub cname: String,
    pub ptrdname: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BatchRecordChange {
    pub id: String,
    pub user_name: String,
    pub user_id: String,
    pub status: String,
    pub comments: String,
    pub created_timestamp: String,
    pub changes: Vec<RecordChange>,
}
