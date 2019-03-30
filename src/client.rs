use crate::api_types::*;
use crate::aws_sign;
use failure::Error;
use log::debug;
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

impl Client {
    pub fn from_env() -> Result<Self, Error> {
        Ok(Client {
            access_key: std::env::var("VINYLDNS_ACCESS_KEY")?,
            secret_key: std::env::var("VINYLDNS_SECRET_KEY")?,
            host: std::env::var("VINYLDNS_HOST")?,
            client: reqwest::Client::new(),
        })
    }

    pub fn new(access_key: String, secret_key: String, host: String) -> Self {
        Client {
            access_key,
            secret_key,
            host,
            client: reqwest::Client::new(),
        }
    }

    pub fn groups(&self) -> Result<Vec<Group>, Error> {
        let groups: Groups = self.request("GET", "/groups", &[])?;
        Ok(groups.groups)
    }

    fn request<R: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
    ) -> Result<R, Error> {
        let mut req =
            reqwest::Request::new(method.parse()?, format!("{}{}", self.host, path).parse()?);

        req.headers_mut()
            .insert(header::CONTENT_TYPE, "application/json".parse()?);

        let ctime = aws_sign::Utc::now();
        {
            use chrono::{Datelike, Timelike};
            use sha2::{Digest, Sha256};

            let fallback_host = req.url().domain().unwrap().to_owned();
            let fallback_content_type = "application/x-www-form-urlencoded; charset=UTF-8";
            let fallback_date = &format!(
                "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
                ctime.year(),
                ctime.month(),
                ctime.day(),
                ctime.hour(),
                ctime.minute(),
                ctime.second()
            );

            let headers = req.headers_mut();

            if !headers.contains_key(header::HOST) {
                headers.insert(header::HOST, fallback_host.parse()?);
            }

            if !headers.contains_key(header::CONTENT_TYPE) {
                headers.insert(header::CONTENT_TYPE, fallback_content_type.parse()?);
            }

            headers.insert("X-Amz-Date", fallback_date.parse()?);
            headers.insert(
                "X-Amz-Content-Sha256",
                format!("{:x}", Sha256::digest(body)).parse()?,
            );
        }

        let auth_val = aws_sign::auth_header(
            req.method(),
            req.url(),
            req.headers(),
            body,
            ctime,
            "us-east-1",
            "s3",
            &self.access_key, //"testUserAccessKey",
            &self.secret_key, //"testUserSecretKey",
        );
        req.headers_mut()
            .insert(header::AUTHORIZATION, auth_val.parse()?);

        let mut res = self.client.execute(req)?;
        let body = res.text()?;
        debug!("{}", body);
        let res: R = serde_json::from_str(&body)?;

        Ok(res)
    }
}
