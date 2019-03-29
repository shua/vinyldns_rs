use reqwest;

mod aws_sign;

fn aws_signv4(req: &mut reqwest::Request) {
    use chrono::{Datelike, Timelike};
    use reqwest::header::HeaderValue;
    use sha2::{Digest, Sha256};

    let ctime = aws_sign::Utc::now();
    let payload = "".as_bytes();
    {
        let fallback_host = HeaderValue::from_str(&req.url().domain().unwrap().to_owned()).unwrap();
        let fallback_content_type =
            HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8");
        let fallback_date = HeaderValue::from_str(&format!(
            "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
            ctime.year(),
            ctime.month(),
            ctime.day(),
            ctime.hour(),
            ctime.minute(),
            ctime.second()
        ))
        .unwrap();
        let headers = req.headers_mut();
        if !headers.contains_key("host") {
            headers.insert("host", fallback_host);
        }

        if !headers.contains_key("content-type") {
            headers.insert("content-type", fallback_content_type);
        }

        headers.insert("X-Amz-Date", fallback_date);
        headers.insert("X-Amz-Content-Sha256", HeaderValue::from_str(&format!("{:x}", Sha256::digest(payload))).unwrap());
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
    req.headers_mut().insert(
        "authorization",
        reqwest::header::HeaderValue::from_str(&auth_val).unwrap(),
    );
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
