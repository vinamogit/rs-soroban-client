use crate::error::*;
use crate::server::*;
use crate::soroban_rpc::GetHealthResponse;
use crate::soroban_rpc::GetHealthWrapperResponse;
use serde::Deserialize;
use serde_json::json;
use wiremock::matchers::method;
use wiremock::matchers::path;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;

#[test]
fn server_new() {
    let s1 = Server::new(
        "https://rpc",
        Options {
            allow_http: None,
            timeout: None,
            headers: None,
        },
    );
    assert!(s1.is_ok(), "https scheme with allow_http None");

    let s2 = Server::new(
        "/rpc",
        Options {
            allow_http: None,
            timeout: None,
            headers: None,
        },
    );
    assert!(matches!(
        s2.err(),
        Some(Error::InvalidRpc(InvalidRpcUrl::NotHttpScheme)),
    ));

    let s3 = Server::new(
        "/rpc",
        Options {
            allow_http: Some(true),
            timeout: None,
            headers: None,
        },
    );
    assert!(matches!(
        s3.err(),
        Some(Error::InvalidRpc(InvalidRpcUrl::NotHttpScheme)),
    ));

    let s4 = Server::new(
        "http://rpc",
        Options {
            allow_http: Some(true),
            timeout: None,
            headers: None,
        },
    );
    assert!(s4.is_ok(), "http scheme with allow_http true");

    let s5 = Server::new(
        "",
        Options {
            allow_http: Some(true),
            timeout: None,
            headers: None,
        },
    );
    assert!(matches!(
        s5.err(),
        Some(Error::InvalidRpc(InvalidRpcUrl::InvalidUri(_))),
    ));

    let s6 = Server::new(
        "http://rpc",
        Options {
            allow_http: Some(false),
            timeout: None,
            headers: None,
        },
    );
    assert!(matches!(
        s6.err(),
        Some(Error::InvalidRpc(InvalidRpcUrl::UnsecureHttpNotAllowed)),
    ));
}

#[tokio::test]
async fn get_health() {
    let mock_server = MockServer::start().await;
    let server_url = mock_server.uri();

    let response = ResponseTemplate::new(200).set_body_raw(
        json!({"jsonrpc": "2.0", "id": 1, "result": {"status": "healthy"}})
            .to_string()
            .as_str(),
        "application/json",
    );

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(response)
        .expect(1..)
        .mount(&mock_server)
        .await;

    let s = Server::new(
        &server_url,
        Options {
            allow_http: Some(true),
            timeout: None,
            headers: None,
        },
    )
    .unwrap();
    let result = s.get_health().await.expect("Should not fail");

    let expect = GetHealthWrapperResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: GetHealthResponse {
            status: "healthy".to_string(),
        },
    };

    assert_eq!(dbg!(result), expect);
}
