use jwt_simple::prelude::*;
use log;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde_json;
use url::form_urlencoded;

static SUBJECT: &str = "fba8ce10-6689-439e-9344-c62cd1a1040f";
static AUDIENCE: &str = "/sso/oauth2/realms/root/realms/api/access_token";
static ISSUER: &str = "fba8ce10-6689-439e-9344-c62cd1a1040f";
static REDIRECT_URL: &str = "/sso/";
static GRANT_TYPE: &str = "client_credentials";
static CLIENT_ASSERTION_TYPE: &str = "urn:ietf:params:oauth:client-assertion-type:jwt-bearer";
static URL: &str = "http://localhost:8080";
static SCOPE_T: &str = "security.ts.bfc8.tokenization";
static SCOPE_D: &str = "security.ts.bfc8.detokenization";
static PRIVATE_KEY_PEM: &str = r#"
-----BEGIN RSA PRIVATE KEY-----
MIIEogIBAAKCAQEA4W8R3BGGQtKuNufJskRY9pNUrIxhfkfVbzmcLm28rmY4YHRq
Vg5PUpzqfCrMFWk5N5385nbQUeFA8rnhPelXGTFGcYZdsoK/ZHIWgUn2y2kpy6af
CYykDRqMeGotahVNlve+/nG94uvOMfDHb3UYP3RTsU3LtM0sQmfjmp9TEZMW/m4q
EBMatTPpUcSR6GOtWmcJjO1Cb6XJXxmVXyDPnG6kiJB4m/c2VeG+3aLAXvea341c
D1Z8guvse3n4U0NLPwXD+eyKLUT2pX3SOf8Xi/8IsGTbfXAEYRJ0zqrBq/AH04a5
b4Z34wT2KjYPlZeQ0iSa6hiqQMb5INaqnoLKrwIDAQABAoIBACXZi7Ka4KN8kxKV
5TgAoL05rGhSI5Zbwjqr/gyPkTPo5sNPGIF9YpUY0ofpeZ2IrronVQxu23g77Tcz
vV0zPdP1gYHTEQ1FRZQ8UYoAnKa3b/SI87/bNLocgIRBM4HRtNbJnvBCsaKVe+fH
qPyOOPWCdwDYcoDXR/yYS88su/pBmtYGe+zNixFd2vpbBmvXo0oVEupnkfBAfq3E
D/AZt5oP8Y5GqrQk4JzscnTHJYuPaMAGQjQDPy9GzdKx8JjDTmsEWgSYwi7Eq9z3
Qjz/Yy5wXb3EWgjJt0IKB900n9F7dN8FaSnWz/PgDrRvcGcxqFAWGr7AgB1AvEkW
MWce1VECgYEA9NuSCbDcCGhZ8usGaAiv7Ma5mU8nN/RXngZUMR01ptHyOaCCK6gy
EBbk4nKNxIx8RRL5mL51JqZNJttB5SyrFUBmzT4e0lVX7MQAgZbvfCrczEtbeYza
8dVbunt23QAPYR97P70g7LAMI7lSE2FjO6v/1aLJ5a5zE1cESFYFjX8CgYEA67E5
fcmD35J/ecObNm22WtsSALspktsCQZUUr87PTlioyyByIfBlDsCwxkmosA3JQVDI
PoXVUHO6j2H9ZJMeMRP+2dzPhr3Kbamgaui/03PxUP42oGph0V+nd9DKCRRDPczr
p99qZj/ZXY5gQrrK/LBFlCJGdCA2/VNMD9LUutECgYBpU5YDQyezGigvHTgZQjti
z60ArJLDOAEEgdZvdhAhHUhjz46O9v/p+d8lJX+kH4ZJNDtXn/3GzVx8rW4wBcHm
F1jhSqp5caqrtzg8hk+oCswFuRi1NYjs5AlcM0XTFPaIocEjjth919CxAUO0AH1u
KrOXEzpj6WGo3RKDhzVGQwKBgDRfjlnwRJiSsjb5i2z91i68TXTSIjva5NZz7UPf
3dsYAnIYxKPcCmjL0rxNM+njNlyIOecC0FLvVH52FNubgXMAcXoyAf97/xZ9QgNf
svim4/HOe48L9K6U5d7PC4YW9ZYvChKkp70O5RtP/v31ChQ6i0uOjj9RxUZnkxhV
DiZxAoGANtbSVfFfEPO5nHqdgeD4zFmOaUImt+1AZ0gtToYTM49b3+KT+sK6dXA/
/9lZJQIfdOl27MZ5TI2j7ckkvB9m/ySNabv8wg9kPthAYN22Y+Dx6imqn2QfwWN+
yJg4Gr37vy7/zwGlQYfNsOFqBd9SmevNzIjktqW7BK5TzoNXztE=
-----END RSA PRIVATE KEY-----
"#;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(AddHeaderRoot)
    });
}}

struct AddHeaderRoot;

impl Context for AddHeaderRoot {}

impl RootContext for AddHeaderRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(AddHeader { context_id }))
    }
}

struct AddHeader {
    context_id: u32,
}

impl Context for AddHeader {
    fn on_http_call_response(
        &mut self,
        _token_id: u32,
        _num_headers: usize,
        body_size: usize,
        _num_trailers: usize,
    ) {
        if let Some(body) = self.get_http_call_response_body(0, body_size) {
            if !body.is_empty() {
                log::info!(
                    "response from jwt issuer: {:?}",
                    String::from_utf8_lossy(&body)
                );
                let payload: ResponsePayload = serde_json::from_slice(&body).unwrap();
                let access_token = payload.access_token;
                self.set_http_request_header(
                    "Authorization",
                    Some(format!("Bearer {access_token}").as_str()),
                );
            }
        } else {
            log::info!("No jwt returned.");
        }

        // we will continue the request no matter what
        self.resume_http_request();
    }
}

impl HttpContext for AddHeader {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        for (name, value) in self.get_http_request_headers() {
            log::info!(
                "context_id: {}: Orig header -> {}: {}",
                self.context_id,
                name,
                value
            );
        }

        log::info!("New header -> From-Proxy-Wasm: Hello");
        self.set_http_request_header("From-Proxy-Wasm", Some("Hello"));

        log::info!("prepare client jwt ...");

        let audience = format!("{}{}", URL, AUDIENCE);
        let redirect_url = format!("{}{}", URL, REDIRECT_URL);

        let claims = Claims::create(Duration::from_mins(1))
            .with_issuer(ISSUER)
            .with_audience(audience)
            .with_subject(SUBJECT);

        let key_pair = RS256KeyPair::from_pem(PRIVATE_KEY_PEM).unwrap();
        let token = key_pair.sign(claims).unwrap();

        log::info!("client token: {token}");

        let payload = form_urlencoded::Serializer::new(String::new())
            .append_pair("grant_type", GRANT_TYPE)
            .append_pair("redirect_uri", &redirect_url)
            .append_pair("scope", SCOPE_T)
            .append_pair("client_assertion_type", CLIENT_ASSERTION_TYPE)
            .append_pair("client_assertion", token.as_str())
            .finish();

        log::info!("request payload: {}", payload);

        self
            .dispatch_http_call(
                "jwtissuer",
                vec![
                    (":method", "POST"),
                    (":path", "/sso/oauth2/realms/root/realms/api/access_token"),
                    (":scheme", "http"),
                    (":authority", "localhost:8080"),
                    ("Content-Type", "application/x-www-form-urlencoded"),
                ],
                Some(payload.as_bytes()),
                vec![],
                std::time::Duration::from_secs(5),
            )
            .unwrap();

        Action::Pause
    }
}

#[derive(Deserialize)]
struct ResponsePayload {
    access_token: String,
    scope: String,
    token_type: String,
    expires_in: u16,
}
