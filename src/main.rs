use reqwest;
use std::str::FromStr;

mod aws_sign;

fn main() {
    use reqwest::Url;
    use std::str::FromStr;

    let req = reqwest::Request::new(
        reqwest::Method::GET,
        Url::from_str("http://localhost:9000/groups").unwrap(),
    );
    println!("{:?}", req);

    let can_req = req.url().scheme().to_owned()
        + "://"
        + req.url().domain().unwrap_or_default()
        + &req
            .url()
            .port()
            .map(|p| format!(":{}", p))
            .unwrap_or_default()
        + req.url().path();
    println!("{:?}", can_req);
}
