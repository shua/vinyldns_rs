use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
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

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ZoneACL {
    pub rules: Vec<ACLRule>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Zone {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub email: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub status: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub created: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub admin_group_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub latest_sync: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub updated: String,
    #[serde(skip_serializing_if = "zone_connection_is_empty")]
    pub connection: ZoneConnection,
    #[serde(skip_serializing_if = "zone_connection_is_empty")]
    pub transfer_connection: ZoneConnection,
    pub acl: ZoneACL,

    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_test: bool,
}

fn zone_connection_is_empty(zc: &ZoneConnection) -> bool {
    zc.key.is_empty()
        && zc.key_name.is_empty()
        && zc.name.is_empty()
        && zc.primary_server.is_empty()
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ZoneResponse {
    pub zone: Zone,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ZoneUpdateResponse {
    pub zone: Zone,
    pub user_id: String,
    pub change_type: String,
    pub status: String,
    pub created: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Zones {
    pub zones: Vec<Zone>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ZoneChanges {
    #[serde(default)]
    pub zone_id: String,
    pub zone_changes: Vec<ZoneChange>,
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
pub struct RecordSetChanges {
    pub record_set_changes: Vec<RecordSetChange>,
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

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RecordSet {
    pub id: String,
    pub zone_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_set_type: String,
    pub status: String,
    pub created: String,
    pub updated: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preference: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsdname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ptrdname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub record_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
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

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Group {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub email: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub status: String,
    #[serde(skip_serializing_if = "String::is_empty")]
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
