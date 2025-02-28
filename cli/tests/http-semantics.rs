//! Tests related to HTTP semantics (e.g. framing headers, status codes).

mod common;

use {
    common::{Test, TestResult},
    hyper::{header, Request, Response, StatusCode},
};

#[tokio::test(flavor = "multi_thread")]
async fn framing_headers_are_overridden() -> TestResult {
    // Set up the test harness
    let test = Test::using_fixture("bad-framing-headers.wasm")
        // The "TheOrigin" backend checks framing headers on the request and then echos its body.
        .backend("TheOrigin", "http://127.0.0.1:9000/")
        .host(9000, |req| {
            assert!(!req.headers().contains_key(header::TRANSFER_ENCODING));
            assert_eq!(
                req.headers().get(header::CONTENT_LENGTH),
                Some(&hyper::header::HeaderValue::from(9))
            );
            Response::new(Vec::from(&b"salutations"[..]))
        });

    let resp = test
        .via_hyper()
        .against(
            Request::post("http://127.0.0.1:7878")
                .body("greetings")
                .unwrap(),
        )
        .await;

    assert_eq!(resp.status(), StatusCode::OK);

    assert!(!resp.headers().contains_key(header::TRANSFER_ENCODING));
    assert_eq!(
        resp.headers().get(header::CONTENT_LENGTH),
        Some(&hyper::header::HeaderValue::from(11))
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn content_length_is_computed_correctly() -> TestResult {
    // Set up the test harness
    let test = Test::using_fixture("content-length.wasm")
        // The "TheOrigin" backend supplies a fixed-size body.
        .backend("TheOrigin", "http://127.0.0.1:9000/")
        .host(9000, |_| {
            Response::new(Vec::from(&b"ABCDEFGHIJKLMNOPQRST"[..]))
        });

    let resp = test
        .via_hyper()
        .against(Request::get("http://127.0.0.1:7878").body("").unwrap())
        .await;

    assert_eq!(resp.status(), StatusCode::OK);

    assert!(!resp.headers().contains_key(header::TRANSFER_ENCODING));
    assert_eq!(
        resp.headers().get(header::CONTENT_LENGTH),
        Some(&hyper::header::HeaderValue::from(28))
    );
    let resp_body = resp.into_body().read_into_string().await.unwrap();
    assert_eq!(resp_body, "ABCD12345xyzEFGHIJKLMNOPQRST");

    Ok(())
}
