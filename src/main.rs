#![allow(dead_code, unused_imports)]

use clap::{clap_app, crate_version, SubCommand};
use log::{debug, info};
use reqwest;

mod api_types;
mod aws_sign;
mod client;

fn aws_signv4(req: &mut reqwest::Request) {
    use chrono::{Datelike, Timelike};
    use reqwest::header;
    use reqwest::header::HeaderValue;
    use sha2::{Digest, Sha256};

    let ctime = aws_sign::Utc::now();
    let payload = "".as_bytes();
    {
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
            headers.insert(header::HOST, fallback_host.parse().unwrap());
        }

        if !headers.contains_key(header::CONTENT_TYPE) {
            headers.insert(header::CONTENT_TYPE, fallback_content_type.parse().unwrap());
        }

        headers.insert("X-Amz-Date", fallback_date.parse().unwrap());
        headers.insert(
            "X-Amz-Content-Sha256",
            HeaderValue::from_str(&format!("{:x}", Sha256::digest(payload))).unwrap(),
        );
    }

    let auth_val = aws_sign::auth_header(
        req.method(),
        req.url(),
        req.headers(),
        payload,
        ctime,
        "us-east-1",
        "s3",
        "testUserAccessKey",
        "testUserSecretKey",
    );
    req.headers_mut()
        .insert(header::AUTHORIZATION, auth_val.parse().unwrap());
}

fn f<O: serde::Serialize>(x: Result<O, failure::Error>) -> String {
    match x {
        Ok(x) => format!("{}", serde_json::to_string_pretty(&x).unwrap()),
        Err(x) => format!("{}", x),
    }
}

fn g(m: &clap::ArgMatches<'_>, v: &str) -> String {
    m.value_of(v).unwrap_or_default().into()
}

fn main() {
    env_logger::init();

    let matches = clap_app!(("vinyldns-client") =>
        (@setting SubcommandRequiredElseHelp)
        (version: crate_version!())
        // HACK: currently, (@subcommand name-with-hyphen => ...) or (@subcommand ("name-with-hyphen") => ...) won't parse
        // https://github.com/clap-rs/clap/pull/1523
        (subcommand: SubCommand::with_name("list-groups").alias("lg"))
        (subcommand: clap_app!{ @app (SubCommand::with_name("create-group"))
            (alias: "cg")
            (@arg name: -n --name * +takes_value "")
            (@arg email: -e --email * +takes_value "")
            (@arg description: -d --description +takes_value "")
        })
        (subcommand: clap_app!{ @app (SubCommand::with_name("delete-group"))
            (alias: "dg")
            (@arg id: -i --id * +takes_value "")
        })
        (subcommand: SubCommand::with_name("list-zones").alias("lz"))
        (subcommand: clap_app!{ @app (SubCommand::with_name("create-zone"))
            (alias: "cz")
            (@arg name: -n --name * +takes_value "")
            (@arg email: -e --email * +takes_value "")
            (@arg ("admin-group-id"): -a --("admin-group-id") * +takes_value "")
        })
        (subcommand: clap_app!{ @app (SubCommand::with_name("get-record-sets"))
            (alias: "gr")
            (@arg id: -i --id * +takes_value "")
        })
    )
    .get_matches();

    let client = client::Client::from_env().unwrap();

    let out = match matches.subcommand() {
        ("list-groups", _) => f(client.groups()),
        ("create-group", Some(matches)) => f(client.group_create(&api_types::Group {
            name: g(matches, "name"),
            email: g(matches, "email"),
            description: g(matches, "description"),
            ..std::default::Default::default()
        })),
        ("delete-group", Some(matches)) => f(client.group_delete(&g(matches, "id"))),
        ("list-zones", _) => f(client.zones()),
        ("create-zone", Some(matches)) => f(client.zone_create(&api_types::Zone {
            name: g(matches, "name"),
            email: g(matches, "email"),
            admin_group_id: g(matches, "admin-group-id"),
            is_test: true,
            ..std::default::Default::default()
        })),
        ("get-record-sets", Some(matches)) => f(client.record_sets(&g(matches, "id"))),
        _ => unimplemented!(),
    };

    println!("{}", out);
}
