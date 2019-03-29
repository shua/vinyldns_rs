use reqwest;

mod aws_sign;

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
    println!("Authorization: {}", auth_val);
    req.headers_mut()
        .insert(header::AUTHORIZATION, auth_val.parse().unwrap());
}

fn main() {
    use reqwest::Url;
    use std::str::FromStr;

    let mut req = reqwest::Request::new(
        reqwest::Method::GET,
        Url::from_str("http://localhost:9000/").unwrap(),
    );

    aws_signv4(&mut req);
    println!("{:?}", req);

    let client = reqwest::Client::new();
    let mut res = client.execute(req).unwrap();
    println!("{:?}", res);
    println!("{:?}", res.text().unwrap());
}
