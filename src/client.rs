#![allow(dead_code)]

use crate::api_types::*;
use crate::aws_sign;
use failure::Fail;
use log::*;
use reqwest;
use reqwest::header;
use serde::de::DeserializeOwned;
use serde_json;

pub struct Client {
    pub access_key: String,
    pub secret_key: String,
    pub host: String,
    client: reqwest::Client,
}

#[derive(Debug, Fail)]
pub enum ClientError {
    #[fail(display = "missing environment variable: {}", _0)]
    MissingEnvironmentVariable(String),
    #[fail(display = "failed to parse value: {}", _0)]
    Parsing(String),
    #[fail(display = "failed to execute request: {}", _0)]
    Http(String),
    #[fail(display = "failed deserializing response: {}\n{}", _0, _1)]
    Deserializing(serde_json::error::Error, String),
}
use crate::client::ClientError::*;

type Result<R> = core::result::Result<R, failure::Error>;

impl Client {
    pub fn from_env() -> Result<Self> {
        fn e(n: &str) -> core::result::Result<String, ClientError> {
            std::env::var(n).map_err(|e| MissingEnvironmentVariable(e.to_string()))
        }
        Ok(Client {
            access_key: e("VINYLDNS_ACCESS_KEY")?,
            secret_key: e("VINYLDNS_SECRET_KEY")?,
            host: e("VINYLDNS_HOST")?,
            client: reqwest::Client::new(),
        })
    }

    pub fn new(access_key: &str, secret_key: &str, host: &str) -> Self {
        Client {
            access_key: access_key.to_string(),
            secret_key: secret_key.to_string(),
            host: host.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn zones(&self) -> Result<Vec<Zone>> {
        let zones: Zones = self.request("GET", "/zones", &[])?;
        Ok(zones.zones)
    }

    pub fn zone(&self, id: &str) -> Result<Zone> {
        let zone: ZoneResponse = self.request("GET", &format!("/zones/{}", id), &[])?;
        Ok(zone.zone)
    }

    pub fn zone_create(&self, zone: &Zone) -> Result<ZoneUpdateResponse> {
        let zone = serde_json::to_string(zone)?;
        let response: ZoneUpdateResponse = self.request("POST", "/zones", &zone.as_bytes())?;
        Ok(response)
    }

    pub fn zone_update(&self, id: &str, zone: &Zone) -> Result<ZoneUpdateResponse> {
        let zone = serde_json::to_string(zone)?;
        let response: ZoneUpdateResponse =
            self.request("PUT", &format!("/zones/{}", id), &zone.as_bytes())?;
        Ok(response)
    }

    pub fn zone_delete(&self, id: &str) -> Result<ZoneUpdateResponse> {
        let response: ZoneUpdateResponse =
            self.request("DELETE", &format!("/zones/{}", id), &[])?;
        Ok(response)
    }

    pub fn zone_changes(&self, id: &str) -> Result<Vec<ZoneChange>> {
        let changes: ZoneChanges = self.request("GET", &format!("/zones/{}/changes", id), &[])?;
        Ok(changes.zone_changes)
    }

    pub fn record_sets(&self, zone_id: &str) -> Result<Vec<RecordSet>> {
        // todo: iterator/stream response
        let response: RecordSetsResponse =
            self.request("GET", &format!("/zones/{}/recordsets", zone_id), &[])?;
        Ok(response.record_sets)
    }

    pub fn record_set(&self, zone_id: &str, id: &str) -> Result<RecordSet> {
        let record_set: RecordSet =
            self.request("GET", &format!("/zones/{}/recordsets/{}", zone_id, id), &[])?;
        Ok(record_set)
    }

    pub fn record_set_create(
        &self,
        zone_id: &str,
        rs: &RecordSet,
    ) -> Result<RecordSetUpdateResponse> {
        let rs = serde_json::to_string(rs)?;
        let response: RecordSetUpdateResponse = self.request(
            "POST",
            &format!("/zones/{}/recordsets", zone_id),
            &rs.as_bytes(),
        )?;
        Ok(response)
    }

    pub fn record_set_update(
        &self,
        zone_id: &str,
        id: &str,
        rs: &RecordSet,
    ) -> Result<RecordSetUpdateResponse> {
        let rs = serde_json::to_string(rs)?;
        let response: RecordSetUpdateResponse = self.request(
            "PUT",
            &format!("/zones/{}/recordsets/{}", zone_id, id),
            &rs.as_bytes(),
        )?;
        Ok(response)
    }

    pub fn record_set_delete(&self, zone_id: &str, id: &str) -> Result<RecordSetUpdateResponse> {
        let response: RecordSetUpdateResponse = self.request(
            "DELETE",
            &format!("/zones/{}/recordsets/{}", zone_id, id),
            &[],
        )?;
        Ok(response)
    }

    pub fn record_set_changes(&self, zone_id: &str) -> Result<Vec<RecordSetChange>> {
        // todo: iterate/stream response
        let changes: RecordSetChanges =
            self.request("GET", &format!("/zones/{}/recordsetchanges", zone_id), &[])?;
        Ok(changes.record_set_changes)
    }

    pub fn record_set_change(
        &self,
        zone_id: &str,
        record_set_id: &str,
        change_id: &str,
    ) -> Result<RecordSetChange> {
        let change: RecordSetChange = self.request(
            "GET",
            &format!(
                "/zones/{}/recordsets/{}/changes/{}",
                zone_id, record_set_id, change_id
            ),
            &[],
        )?;
        Ok(change)
    }

    pub fn groups(&self) -> Result<Vec<Group>> {
        let groups: Groups = self.request("GET", "/groups", &[])?;
        Ok(groups.groups)
    }

    pub fn group_create(&self, group: &Group) -> Result<Group> {
        let group = serde_json::to_string(group)?;
        let group: Group = self.request("POST", "/groups", &group.as_bytes())?;
        Ok(group)
    }

    pub fn group(&self, group_id: &str) -> Result<Group> {
        let group: Group = self.request("GET", &format!("/groups/{}", group_id), &[])?;
        Ok(group)
    }

    pub fn group_delete(&self, group_id: &str) -> Result<Group> {
        let group: Group = self.request("DELETE", &format!("/groups/{}", group_id), &[])?;
        Ok(group)
    }

    pub fn group_update(&self, group_id: &str, group: &Group) -> Result<Group> {
        let group = serde_json::to_string(group)?;
        let group: Group =
            self.request("PUT", &format!("/groups/{}", group_id), &group.as_bytes())?;
        Ok(group)
    }

    pub fn group_admins(&self, group_id: &str) -> Result<Vec<User>> {
        let admins: GroupAdmins =
            self.request("GET", &format!("/groups/{}/admins", group_id), &[])?;
        Ok(admins.admins)
    }

    pub fn group_members(&self, group_id: &str) -> Result<Vec<User>> {
        let members: GroupMembers =
            self.request("GET", &format!("/groups/{}/members", group_id), &[])?;
        Ok(members.members)
    }

    pub fn group_activity(&self, group_id: &str) -> Result<GroupChanges> {
        let activity: GroupChanges =
            self.request("GET", &format!("/groups/{}/activiy", group_id), &[])?;
        Ok(activity)
    }

    fn request<R: DeserializeOwned>(&self, method: &str, path: &str, body: &[u8]) -> Result<R> {
        let dt = aws_sign::Utc::now();

        trace!(
            "{} {}\n{}",
            method,
            path,
            std::str::from_utf8(body).unwrap_or_default()
        );

        let mut req =
            reqwest::Request::new(method.parse()?, format!("{}{}", self.host, path).parse()?);
        req.headers_mut()
            .insert(header::CONTENT_TYPE, "application/json".parse()?);
        aws_sign::prepare_request(&mut req, dt, body);
        let auth_val = aws_sign::auth_header(
            req.method(),
            req.url(),
            req.headers(),
            body,
            dt,
            "us-east-1",
            "s3",
            &self.access_key, //"testUserAccessKey",
            &self.secret_key, //"testUserSecretKey",
        );
        req.headers_mut()
            .insert(header::AUTHORIZATION, auth_val.parse()?);

        trace!("{:?}", req.headers());
        *req.body_mut() = Some(body.to_owned().into());

        let resbody = self
            .client
            .execute(req)
            .map_err(|e| Http(e.to_string()))
            .and_then(|mut res| {
                if !res.status().is_success() {
                    Err(Http(res.text().unwrap_or(res.status().to_string())))
                } else {
                    res.text().map_err(|e| Http(e.to_string()))
                }
            })?;
        let res = serde_json::from_str(&resbody).map_err(|e| Deserializing(e, resbody))?;
        Ok(res)
    }
}
