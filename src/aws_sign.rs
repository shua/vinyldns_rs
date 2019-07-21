pub use chrono::{DateTime, Utc};
pub use reqwest::{header::HeaderMap, Method};

mod task1 {
    // https://docs.aws.amazon.com/general/latest/gr/sigv4-create-canonical-request.html

    use log::trace;
    use reqwest;
    use sha2::{Digest, Sha256};
    fn canonical_uri(url: &reqwest::Url) -> String {
        // 2. The canonical URI is the URI-encoded version of the absolute path component of the URI, which is everything in the URI from the HTTP host to the question mark character ("?") that begins the query string parameters (if any).
        // Normalize URI paths according to RFC 3986. Remove redundant and relative path components. Each path segment must be URI-encoded twice (except for Amazon S3 which only gets URI-encoded once).
        // If the absolute path is empty, use a forward slash (/). In the example IAM request, nothing follows the host in the URI, so the absolute path is empty.
        url.path().to_string() // XXX: there's no normalization happening here, but parsing Url's from strings _should_ do that for us
    }

    fn canonical_query_string(url: &reqwest::Url) -> String {
        // 3. Add the canonical query string, followed by a newline character. If the request does not include a query string, use an empty string (essentially, a blank line). The example request has the following query string.

        // a. Sort the parameter names by character code point in ascending order. Parameters with duplicate names should be sorted by value. For example, a parameter name that begins with the uppercase letter F precedes a parameter name that begins with a lowercase letter b.
        let mut query_pairs = url.query_pairs().collect::<Vec<_>>();
        // PartialEq is implemented for (String, String) as dictionary sort for tuple, and the strings inside
        query_pairs.sort();
        // b. URI-encode each parameter name and value according to the following rules:
        //    Do not URI-encode any of the unreserved characters that RFC 3986 defines: A-Z, a-z, 0-9, hyphen ( - ), underscore ( _ ), period ( . ), and tilde ( ~ ).
        //    Percent-encode all other characters with %XY, where X and Y are hexadecimal characters (0-9 and uppercase A-F). For example, the space character must be encoded as %20 (not using '+', as some encoding schemes do) and extended UTF-8 characters must be in the form %XY%ZA%BC.
        fn percent_encode(s: &str) -> String {
            s.to_string()
                .chars()
                .map(|s| match s as u8 {
                    b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                        format!("{}", s)
                    }
                    _ => format!("%{:02x}", s as u8),
                })
                .collect::<String>()
        }
        let query_pairs = query_pairs
            .iter()
            .map(|(k, v)| (percent_encode(k), percent_encode(v)));
        // c. Build the canonical query string by starting with the first parameter name in the sorted list.
        // d. For each parameter, append the URI-encoded parameter name, followed by the equals sign character (=), followed by the URI-encoded parameter value. Use an empty string for parameters that have no value.
        // e. Append the ampersand character (&) after each parameter value, except for the last value in the list.
        let query_string = query_pairs
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        query_string
    }

    fn is_problematic_header(name: &str) -> bool {
        // used to filter out problematic headers from canonical headers and signed headers
        let name = name.to_lowercase();
        (name.starts_with("x-amz-") && name != "x-amz-date") //|| name == "content-type"
    }

    fn canonical_headers(headers: &reqwest::header::HeaderMap) -> String {
        // 4. Add the canonical headers, followed by a newline character. The canonical headers consist of a list of all the HTTP headers that you are including with the signed request.
        // For HTTP/1.1 requests, you must include the host header at a minimum. Standard headers like content-type are optional.For HTTP/2 requests, you must include the :authority header instead of the host header. Different services might require other headers.
        // To create the canonical headers list, convert all header names to lowercase and remove leading spaces and trailing spaces. Convert sequential spaces in the header value to a single space.
        use regex;
        // Build the canonical headers list by sorting the (lowercase) headers by character code and then iterating through the header names. Construct each header according to the following rules:
        //   Append the lowercase header name followed by a colon.
        //   Append a comma-separated list of values for that header. Do not sort the values in headers that have multiple values.
        //   Append a new line ('\n').
        let mut headers = headers
            .iter()
            .filter(|(k, _v)| !is_problematic_header(k.as_str()))
            .map(|(k, v)| {
                format!(
                    "{}:{}\n",
                    k.as_str().to_lowercase(),
                    regex::Regex::new(r"  *")
                        .unwrap()
                        .replace_all(v.to_str().unwrap().trim(), " ")
                )
            })
            .collect::<Vec<_>>();
        // XXX: not combining multiple instances of header values with comma, so that's eventually going to be an issue...
        headers.sort();
        headers.join("")
    }

    pub fn signed_headers(headers: &reqwest::header::HeaderMap) -> String {
        // 5. Add the signed headers, followed by a newline character. This value is the list of headers that you included in the canonical headers. By adding this list of headers, you tell AWS which headers in the request are part of the signing process and which ones AWS can ignore (for example, any additional headers added by a proxy) for purposes of validating the request.
        // For HTTP/1.1 requests, the host header must be included as a signed header. For HTTP/2 requests that include the :authority header instead of the host header, you must include the :authority header as a signed header. If you include a date or x-amz-date header, you must also include that header in the list of signed headers.
        let mut headers = headers
            .iter()
            .filter(|(k, _v)| !is_problematic_header(k.as_str()))
            .map(|(k, _v)| k.as_str().to_lowercase())
            .collect::<Vec<_>>();
        headers.sort();
        headers.dedup();
        headers.join(";")
    }

    fn hashed_payload(payload: &[u8]) -> String {
        // 6. Use a hash (digest) function like SHA256 to create a hashed value from the payload in the body of the HTTP or HTTPS request. Signature Version 4 does not require that you use a particular character encoding to encode text in the payload. However, some AWS services might require a specific encoding. For more information, consult the documentation for that service.
        format!("{:x}", Sha256::digest(payload))
    }

    fn canonical_request(
        method: &reqwest::Method,
        url: &reqwest::Url,
        headers: &reqwest::header::HeaderMap,
        payload: &[u8],
    ) -> String {
        /*
        CanonicalRequest =
            HTTPRequestMethod + '\n' +
            CanonicalURI + '\n' +
            CanonicalQueryString + '\n' +
            CanonicalHeaders + '\n' +
            SignedHeaders + '\n' +
            HexEncode(Hash(RequestPayload))
        */
        // 1. Start with the HTTP request method (GET, PUT, POST, etc.), followed by a newline character.
        let http_request_method = method.as_str().to_string();
        let canonical_uri = canonical_uri(url);
        let canonical_query_string = canonical_query_string(url);
        let canonical_headers = canonical_headers(headers);
        let signed_headers = signed_headers(headers);
        let hashed_payload = hashed_payload(payload);

        // 7. To construct the finished canonical request, combine all the components from each step as a single string. As noted, each component ends with a newline character. If you follow the canonical request pseudocode explained earlier, the resulting canonical request is shown in the following example.
        let canonical_request = http_request_method
            + "\n"
            + &canonical_uri
            + "\n"
            + &canonical_query_string
            + "\n"
            + &canonical_headers
            + "\n"
            + &signed_headers
            + "\n"
            + &hashed_payload;

        trace!("CANONICAL_REQUEST:\n'{}'", canonical_request);

        canonical_request
    }

    pub fn hashed_canonical_request(
        method: &reqwest::Method,
        url: &reqwest::Url,
        headers: &reqwest::header::HeaderMap,
        payload: &[u8],
    ) -> String {
        let hashed_canonical_request = format!(
            "{:x}",
            Sha256::digest(canonical_request(method, url, headers, payload).as_bytes())
        );
        trace!("HASHED_CANONICAL_REQUEST: {}", hashed_canonical_request);
        hashed_canonical_request
    }

    #[cfg(test)]
    mod test {
        use super::*;
        #[test]
        fn test_canonical_uri() {
            assert_eq!(
                canonical_uri(
                    &"https://iam.amazonaws.com/?Action=ListUsers&Version=2010-05-08"
                        .parse()
                        .unwrap()
                ),
                "/"
            );
            assert_eq!(
                canonical_uri(&"http://example.com/something?else".parse().unwrap()),
                "/something"
            );
            assert_eq!(
                canonical_uri(&"http://user:pass@example.com".parse().unwrap()),
                "/"
            );
            assert_eq!(canonical_uri(&"http://x.com/path/..".parse().unwrap()), "/");
        }

        #[test]
        fn test_canonical_query_string() {
            assert_eq!(
                canonical_query_string(
                    &"http://x.com?a=b&b&b=c&c=azAZ09~-_&^val$=' '"
                        .parse()
                        .unwrap()
                ),
                "%5eval%24=%27%20%27&a=b&b=&b=c&c=azAZ09~-_"
            )
        }

        fn build_headers(hdrs: Vec<&'static str>) -> reqwest::header::HeaderMap {
            use std::iter::FromIterator;
            let headers = reqwest::header::HeaderMap::from_iter(hdrs.into_iter().map(|s| {
                let i = s.find(":").unwrap();
                let k = &s[..i];
                let v = &s[i + 1..];
                (
                    reqwest::header::HeaderName::from_static(k),
                    reqwest::header::HeaderValue::from_static(v),
                )
            }));
            headers
        }

        #[test]
        fn test_canonical_headers_simple() {
            let headers = build_headers(vec![
                "host: example.com",
                "special-header:   special   value   ",
            ]);
            assert_eq!(
                canonical_headers(&headers),
                "host:example.com\nspecial-header:special value\n"
            );
        }

        #[test]
        fn test_canonical_headers_multiple_values() {
            let headers = build_headers(vec![
                "host: example.com",
                "special-header:  special    value  ",
                "content-type: thingey",
                "special-header: other  value",
                "special-header: z",
            ]);
            assert_eq!(
                canonical_headers(&headers),
                "host:example.com\nspecial-header:special value\n"
            );
        }

        #[test]
        fn test_signed_headers_simple() {
            let headers = build_headers(vec![
                "host: example.com",
                "special-header:   special   value   ",
            ]);
            assert_eq!(signed_headers(&headers), "host;special-header")
        }

        #[test]
        fn test_signed_headers_multiple_values() {
            let headers = build_headers(vec![
                "host: example.com",
                "special-header:  special    value  ",
                "content-type: thingey",
                "special-header: other  value",
                "special-header: z",
            ]);
            assert_eq!(signed_headers(&headers), "host;special-header")
        }

        #[test]
        fn test_hashed_payload_sha256() {
            assert_eq!(
                hashed_payload(b""),
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            );
        }

        #[test]
        fn test_canonical_request() {
            assert_eq!(
                canonical_request(
                    &reqwest::Method::GET,
                    &"https://iam.amazonaws.com/?Action=ListUsers&Version=2010-05-08"
                        .parse()
                        .unwrap(),
                    &build_headers(vec![
                        "host: iam.amazonaws.com",
                        "x-amz-date:20150830T123600Z",
                        "content-type: application/x-www-form-urlencoded; charset=utf-8",
                    ]),
                    b""
                ),
                r#"GET
/
Action=ListUsers&Version=2010-05-08
host:iam.amazonaws.com
x-amz-date:20150830T123600Z

host;x-amz-date
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"#
            );
        }

        #[test]
        fn test_hashed_canonical_request() {
            let hash = hashed_canonical_request(
                &reqwest::Method::GET,
                &"https://iam.amazonaws.com/?Action=ListUsers&Version=2010-05-08"
                    .parse()
                    .unwrap(),
                &build_headers(vec![
                    "host: iam.amazonaws.com",
                    "x-amz-date:20150830T123600Z",
                    "content-type: application/x-www-form-urlencoded; charset=utf-8",
                ]),
                b"",
            );
            assert_eq!(
                hash,
                "5599feeca6d065c7c80025038896f3f7f008849eacf307aa7d0cf8be7116cea6"
            );
        }
    }
}

mod task2 {
    use chrono::{DateTime, Datelike, Timelike, Utc};
    use log::trace;

    pub fn string_to_sign(
        datetime: DateTime<Utc>,
        region: &str,
        service: &str,
        hashed_canonical_request: &str,
    ) -> String {
        /*
        StringToSign =
            Algorithm + \n +
            RequestDateTime + \n +
            CredentialScope + \n +
            HashedCanonicalRequest
        */

        // 1. Start with the algorithm designation, followed by a newline character. This value is the hashing algorithm that you use to calculate the digests in the canonical request. For SHA256, AWS4-HMAC-SHA256 is the algorithm.
        let algorithm_str = "AWS4-HMAC-SHA256".to_string();
        let request_datetime = request_datetime(datetime);
        let credential_scope = credential_scope(datetime, region, service);

        let string_to_sign = algorithm_str
            + "\n"
            + &request_datetime
            + "\n"
            + &credential_scope
            + "\n"
            + hashed_canonical_request;

        trace!("STRING_TO_SIGN: '{}'", string_to_sign);

        string_to_sign
    }

    fn request_datetime(datetime: DateTime<Utc>) -> String {
        // 2. Append the request date value, followed by a newline character. The date is specified with ISO8601 basic format in the x-amz-date header in the format YYYYMMDD'T'HHMMSS'Z'. This value must match the value you used in any previous steps.
        datetime.format("%Y%m%dT%H%M%SZ").to_string()
    }

    pub fn credential_scope(datetime: DateTime<Utc>, region: &str, service: &str) -> String {
        // 3. Append the credential scope value, followed by a newline character. This value is a string that includes the date, the region you are targeting, the service you are requesting, and a termination string ("aws4_request") in lowercase characters. The region and service name strings must be UTF-8 encoded.
        format!(
            "{}/{}/{}/aws4_request",
            datetime.format("%Y%m%d").to_string(),
            region,
            service
        )
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use chrono::{TimeZone, Utc};
        #[test]
        fn test_request_datetime() {
            assert_eq!(
                request_datetime(Utc.ymd(2015, 8, 30).and_hms(12, 36, 0)),
                "20150830T123600Z"
            );
        }

        #[test]
        fn test_credential_scope() {
            assert_eq!(
                credential_scope(Utc.ymd(2015, 8, 30).and_hms(12, 36, 0), "us-east-1", "iam"),
                "20150830/us-east-1/iam/aws4_request"
            );
        }

        #[test]
        fn test_string_to_sign() {
            assert_eq!(
                string_to_sign(
                    Utc.ymd(2015, 8, 30).and_hms(12, 36, 0),
                    "us-east-1",
                    "iam",
                    "f536975d06c0309214f805bb90ccff089219ecd68b2577efef23edd43b7e1a59"
                ),
                r#"AWS4-HMAC-SHA256
20150830T123600Z
20150830/us-east-1/iam/aws4_request
f536975d06c0309214f805bb90ccff089219ecd68b2577efef23edd43b7e1a59"#
            )
        }
    }
}

mod task3 {
    use chrono::{Date, Datelike, Utc};
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    fn hmac<'a>(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut hmac = HmacSha256::new_varkey(key).unwrap();
        hmac.input(data);
        hmac.result().code().as_slice().to_owned()
    }

    fn sign(k_secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
        /*
        kSecret = your secret access key
        kDate = HMAC("AWS4" + kSecret, Date)
        kRegion = HMAC(kDate, Region)
        kService = HMAC(kRegion, Service)
        kSigning = HMAC(kService, "aws4_request")
        */
        let k_date = hmac(format!("AWS4{}", k_secret).as_bytes(), date.as_bytes());
        let k_region = hmac(k_date.as_slice(), region.as_bytes());
        let k_service = hmac(k_region.as_slice(), service.as_bytes());
        let k_signing = hmac(k_service.as_slice(), "aws4_request".as_bytes());

        k_signing
    }

    pub fn signature(
        secret: &str,
        dt: Date<Utc>,
        region: &str,
        service: &str,
        string_to_sign: &str,
    ) -> String {
        hmac(
            &sign(
                secret,
                &format!("{:04}{:02}{:02}", dt.year(), dt.month(), dt.day()),
                region,
                service,
            ),
            string_to_sign.as_bytes(),
        )
        .into_iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use chrono::{TimeZone, Utc};

        #[test]
        fn test_sign() {
            assert_eq!(
                sign(
                    "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
                    "20150830",
                    "us-east-1",
                    "iam"
                )
                .into_iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>(),
                "c4afb1cc5771d871763a393e44b703571b55cc28424d1a5e86da6ed3c154a4b9"
            );
        }

        #[test]
        fn test_signature() {
            assert_eq!(
                signature(
                    "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
                    Utc.ymd(2015, 8, 30),
                    "us-east-1",
                    "iam",
                    r#"AWS4-HMAC-SHA256
20150830T123600Z
20150830/us-east-1/iam/aws4_request
f536975d06c0309214f805bb90ccff089219ecd68b2577efef23edd43b7e1a59"#
                ),
                "5d672d79c15b13162d9279b0855cfba6789a8edb4c82c400e06b5924a6f2b5d7"
            )
        }
    }
}

pub fn prepare_request(request: &mut reqwest::Request, dt: DateTime<Utc>, body: &[u8]) {
    use reqwest::header;

    let fallback_host = request.url().domain().unwrap().to_owned();
    let fallback_content_type = "application/x-www-form-urlencoded; charset=utf-8";
    let fallback_date = &dt.format("%Y%m%dT%H%M%SZ").to_string();

    {
        use sha2::{Digest, Sha256};
        let headers = request.headers_mut();

        if !headers.contains_key(header::HOST) {
            headers.insert(header::HOST, fallback_host.parse().unwrap());
        }

        if !headers.contains_key(header::CONTENT_TYPE) {
            headers.insert(header::CONTENT_TYPE, fallback_content_type.parse().unwrap());
        }

        headers.insert("X-Amz-Date", fallback_date.parse().unwrap());
        headers.insert(
            "X-Amz-Content-Sha256",
            format!("{:x}", Sha256::digest(body)).parse().unwrap(),
        );
    }
}

pub fn auth_header(
    method: &reqwest::Method,
    url: &reqwest::Url,
    headers: &reqwest::header::HeaderMap,
    payload: &[u8],
    dt: DateTime<Utc>,
    region: &str,
    service: &str,
    access_key_id: &str,
    secret: &str,
) -> String {
    // Authorization: algorithm Credential=access key ID/credential scope, SignedHeaders=SignedHeaders, Signature=signature
    let algorithm = "AWS4-HMAC-SHA256";
    let hashed_canonical_request = task1::hashed_canonical_request(method, url, headers, payload);
    let string_to_sign = task2::string_to_sign(dt, region, service, &hashed_canonical_request);
    let signature = task3::signature(secret, dt.date(), region, service, &string_to_sign);

    let signed_headers = task1::signed_headers(headers);
    let credential_scope = task2::credential_scope(dt, region, service);
    let auth_value = format!(
        "{} Credential={}/{}, SignedHeaders={}, Signature={}",
        algorithm, access_key_id, credential_scope, signed_headers, signature
    );

    auth_value
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;

    fn build_headers(hdrs: Vec<&'static str>) -> reqwest::header::HeaderMap {
        use std::iter::FromIterator;
        let headers = reqwest::header::HeaderMap::from_iter(hdrs.into_iter().map(|s| {
            let i = s.find(":").unwrap();
            let k = &s[..i];
            let v = &s[i + 1..];
            (
                reqwest::header::HeaderName::from_static(k),
                reqwest::header::HeaderValue::from_static(v),
            )
        }));
        headers
    }

    #[test]
    fn test_auth_header() {
        let expected = "Authorization: AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20150830/us-east-1/iam/aws4_request, SignedHeaders=host;x-amz-date, Signature=b2e4af44cfad96d9ffa3c5653674a927b9b0995c33de22e1f843745ce37c1d5e";
        let auth_val = auth_header(
            &reqwest::Method::GET,
            &"https://iam.amazonaws.com/?Action=ListUsers&Version=2010-05-08"
                .parse()
                .unwrap(),
            &build_headers(vec![
                "host: iam.amazonaws.com",
                "x-amz-date:20150830T123600Z",
                "content-type: application/x-www-form-urlencoded; charset=utf-8",
            ]),
            b"",
            Utc.ymd(2015, 8, 30).and_hms(12, 36, 0),
            "us-east-1",
            "iam",
            "AKIDEXAMPLE",
            "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
        );

        assert_eq!(format!("Authorization: {}", auth_val), expected);
    }
}
