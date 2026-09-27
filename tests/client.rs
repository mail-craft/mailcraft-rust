use mailcraft::{Error, MailCraft, SendEmailParams, TemplateParams, UpsertContactParams};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

/// What the fake server received.
struct Received {
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Value,
}

impl Received {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// Serves one request with the given response, returning the client and a handle to what was received.
async fn serve_once(
    status: u16,
    response: &'static str,
) -> (MailCraft, tokio::task::JoinHandle<Received>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(socket);

        let mut request_line = String::new();
        reader.read_line(&mut request_line).await.unwrap();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap().to_string();
        let target = parts.next().unwrap().to_string();

        let mut headers = Vec::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            if line == "\r\n" {
                break;
            }
            let (key, value) = line.split_once(':').unwrap();
            headers.push((key.to_lowercase(), value.trim().to_string()));
        }

        let length: usize = headers
            .iter()
            .find(|(key, _)| key == "content-length")
            .map(|(_, value)| value.parse().unwrap())
            .unwrap_or(0);
        let mut body = vec![0; length];
        reader.read_exact(&mut body).await.unwrap();

        let reply = format!(
            "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
            response.len()
        );
        reader.get_mut().write_all(reply.as_bytes()).await.unwrap();

        Received {
            method,
            target,
            headers,
            body: serde_json::from_slice(&body).unwrap_or(Value::Null),
        }
    });

    let client = MailCraft::builder("mc_test_key")
        .base_url(format!("http://{address}/v1/"))
        .build();

    (client, handle)
}

#[tokio::test]
async fn sends_an_email() {
    let (client, server) = serve_once(202, r#"{"data":{"id":"em_1","status":"queued"}}"#).await;

    let result = client
        .emails()
        .send(SendEmailParams {
            from: "hello@yourdomain.com".into(),
            to: vec!["person@example.com".into()],
            subject: "Welcome!".into(),
            html: Some("<p>Hi</p>".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    let received = server.await.unwrap();
    assert_eq!(received.method, "POST");
    assert_eq!(received.target, "/v1/emails");
    assert_eq!(received.header("authorization"), Some("Bearer mc_test_key"));
    assert_eq!(
        received.header("user-agent"),
        Some(concat!("mailcraft-rust/", env!("CARGO_PKG_VERSION")))
    );
    assert_eq!(received.body["to"][0], "person@example.com");
    assert!(
        received.body.get("text").is_none(),
        "None fields should be omitted"
    );
    assert_eq!(result["data"]["id"], "em_1");
}

#[tokio::test]
async fn list_sends_limit() {
    let (client, server) = serve_once(200, r#"{"data":[]}"#).await;

    client.contacts().list(Some(5)).await.unwrap();

    assert_eq!(server.await.unwrap().target, "/v1/contacts?limit=5");
}

#[tokio::test]
async fn validate_escapes_the_email() {
    let (client, server) = serve_once(200, r#"{"valid":true}"#).await;

    client.emails().validate("a+b@example.com").await.unwrap();

    assert_eq!(
        server.await.unwrap().target,
        "/v1/emails/validate?email=a%2Bb%40example.com"
    );
}

#[tokio::test]
async fn template_update_uses_patch_with_only_given_fields() {
    let (client, server) = serve_once(200, r#"{"data":{"id":3}}"#).await;

    client
        .templates()
        .update(
            3,
            TemplateParams {
                subject: Some("New".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let received = server.await.unwrap();
    assert_eq!(received.method, "PATCH");
    assert_eq!(received.target, "/v1/templates/3");
    assert_eq!(received.body, serde_json::json!({ "subject": "New" }));
}

#[tokio::test]
async fn removes_a_contact_from_a_list() {
    let (client, server) = serve_once(204, "").await;

    client.contacts().remove_from_list("c 1", 2).await.unwrap();

    let received = server.await.unwrap();
    assert_eq!(received.method, "DELETE");
    assert_eq!(received.target, "/v1/contacts/c%201/lists/2");
}

#[tokio::test]
async fn metrics_send_the_date_range() {
    let (client, server) = serve_once(200, r#"{"data":[]}"#).await;

    client
        .metrics()
        .get(Some("2026-01-01"), Some("2026-01-31"))
        .await
        .unwrap();

    assert_eq!(
        server.await.unwrap().target,
        "/v1/metrics?start_date=2026-01-01&end_date=2026-01-31"
    );
}

#[tokio::test]
async fn template_folders_use_the_hyphenated_path() {
    let (client, server) = serve_once(201, r#"{"data":{"id":1}}"#).await;

    client
        .template_folders()
        .create("Onboarding")
        .await
        .unwrap();

    let received = server.await.unwrap();
    assert_eq!(received.target, "/v1/template-folders");
    assert_eq!(received.body["name"], "Onboarding");
}

#[tokio::test]
async fn business_rule_errors() {
    let (client, _server) = serve_once(
        402,
        r#"{"error":{"type":"plan_limit_reached","message":"Monthly limit reached."}}"#,
    )
    .await;

    let error = client.campaigns().send(4).await.unwrap_err();

    match error {
        Error::Api {
            status,
            message,
            error_type,
            ..
        } => {
            assert_eq!(status, 402);
            assert_eq!(message, "Monthly limit reached.");
            assert_eq!(error_type.as_deref(), Some("plan_limit_reached"));
        }
        other => panic!("expected an API error, got {other:?}"),
    }
}

#[tokio::test]
async fn validation_errors() {
    let (client, _server) = serve_once(
        422,
        r#"{"message":"The email field is required.","errors":{"email":["The email field is required."]}}"#,
    )
    .await;

    let error = client
        .contacts()
        .upsert(UpsertContactParams::default())
        .await
        .unwrap_err();

    assert_eq!(error.status(), Some(422));
    match error {
        Error::Api {
            error_type, errors, ..
        } => {
            assert!(error_type.is_none());
            assert_eq!(errors["email"], vec!["The email field is required."]);
        }
        other => panic!("expected an API error, got {other:?}"),
    }
}

#[tokio::test]
async fn unparseable_error_bodies_fall_back_to_the_status() {
    let (client, _server) = serve_once(502, "<html>Bad gateway</html>").await;

    let error = client.lists().list().await.unwrap_err();

    assert_eq!(
        error.to_string(),
        "MailCraft API error 502: Request failed with status 502"
    );
}

#[test]
#[should_panic(expected = "An API key is required")]
fn requires_an_api_key() {
    MailCraft::new("");
}
