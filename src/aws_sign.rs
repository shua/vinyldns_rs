
mod task1 {
    // https://docs.aws.amazon.com/general/latest/gr/sigv4-create-canonical-request.html

    use reqwest;
    use sha2::{Digest, Sha256};
    fn canonical_uri(url: &reqwest::Url) -> String {
        // 2. The canonical URI is the URI-encoded version of the absolute path component of the URI, which is everything in the URI from the HTTP host to the question mark character ("?") that begins the query string parameters (if any).
        // Normalize URI paths according to RFC 3986. Remove redundant and relative path components. Each path segment must be URI-encoded twice (except for Amazon S3 which only gets URI-encoded once).
        // If the absolute path is empty, use a forward slash (/). In the example IAM request, nothing follows the host in the URI, so the absolute path is empty.
        url.path().to_string() // XXX: there's no normalization happening here, but parsing Url's from strings _should_ do that for us
        + "\n"
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

        query_string + "\n"
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
        headers.join("") + "\n"
    }

    fn signed_headers(headers: &reqwest::header::HeaderMap) -> String {
        // 5. Add the signed headers, followed by a newline character. This value is the list of headers that you included in the canonical headers. By adding this list of headers, you tell AWS which headers in the request are part of the signing process and which ones AWS can ignore (for example, any additional headers added by a proxy) for purposes of validating the request.
        // For HTTP/1.1 requests, the host header must be included as a signed header. For HTTP/2 requests that include the :authority header instead of the host header, you must include the :authority header as a signed header. If you include a date or x-amz-date header, you must also include that header in the list of signed headers.
        let mut headers = headers
            .iter()
            .map(|(k, _v)| k.as_str().to_lowercase())
            .collect::<Vec<_>>();
        headers.sort();
        headers.dedup();
        headers.join(";") + "\n"
    }

    fn hashed_payload(payload: &[u8]) -> String {
        // 6. Use a hash (digest) function like SHA256 to create a hashed value from the payload in the body of the HTTP or HTTPS request. Signature Version 4 does not require that you use a particular character encoding to encode text in the payload. However, some AWS services might require a specific encoding. For more information, consult the documentation for that service.
        hash(payload)
    }

    pub fn hash(payload: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.input(payload);
        hasher
            .result()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
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
        let http_request_method = method.as_str().to_string() + "\n";

        let canonical_uri = canonical_uri(url);
        println!("canonical_uri: {:?}", canonical_uri);

        let canonical_query_string = canonical_query_string(url);
        println!("canonical_query_string: {:?}", canonical_query_string);

        let canonical_headers = canonical_headers(headers);
        println!("canonical_headers: {:?}", canonical_headers);

        let signed_headers = signed_headers(headers);
        println!("signed_headers: {:?}", signed_headers);

        let hashed_payload = hashed_payload(payload);
        println!("hashed_payload: {:?}", hashed_payload);

        // 7. To construct the finished canonical request, combine all the components from each step as a single string. As noted, each component ends with a newline character. If you follow the canonical request pseudocode explained earlier, the resulting canonical request is shown in the following example.
        let canonical_request = http_request_method
            + &canonical_uri
            + &canonical_query_string
            + &canonical_headers
            + &signed_headers
            + &hashed_payload;
        println!("canonical_request:\n{}", canonical_request);

        canonical_request
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
                "/\n"
            );
            assert_eq!(
                canonical_uri(&"http://example.com/something?else".parse().unwrap()),
                "/something\n"
            );
            assert_eq!(
                canonical_uri(&"http://user:pass@example.com".parse().unwrap()),
                "/\n"
            );
            assert_eq!(
                canonical_uri(&"http://x.com/path/..".parse().unwrap()),
                "/\n"
            );
        }

        #[test]
        fn test_canonical_query_string() {
            assert_eq!(
                canonical_query_string(
                    &"http://x.com?a=b&b&b=c&c=azAZ09~-_&^val$=' '"
                        .parse()
                        .unwrap()
                ),
                "%5eval%24=%27%20%27&a=b&b=&b=c&c=azAZ09~-_\n"
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
                "host:example.com\nspecial-header:special value\n\n"
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
                "host:example.com\nspecial-header:special value\n\n"
            );
        }

        #[test]
        fn test_signed_headers_simple() {
            let headers = build_headers(vec![
                "host: example.com",
                "special-header:   special   value   ",
            ]);
            assert_eq!(signed_headers(&headers), "host;special-header\n")
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
            assert_eq!(
                signed_headers(&headers),
                "content-type;host;special-header\n"
            )
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
content-type:application/x-www-form-urlencoded; charset=utf-8
host:iam.amazonaws.com
x-amz-date:20150830T123600Z

content-type;host;x-amz-date
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"#
            );
        }

        #[test]
        fn test_hased_canonical_request() {
            let req = canonical_request(
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
            let hash = hash(req.as_bytes());
            assert_eq!(
                hash,
                "f536975d06c0309214f805bb90ccff089219ecd68b2577efef23edd43b7e1a59"
            );
        }
    }
}
