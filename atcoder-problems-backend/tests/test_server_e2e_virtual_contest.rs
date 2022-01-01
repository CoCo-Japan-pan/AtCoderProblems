use actix_web::Result;
use async_trait::async_trait;
use atcoder_problems_backend::server::{run_server, Authentication, GitHubUserResponse};
use rand::Rng;
use serde_json::{json, Value};

pub mod utils;

#[derive(Clone)]
struct MockAuth;

const VALID_CODE: &str = "VALID-CODE";
const VALID_TOKEN: &str = "VALID-TOKEN";

#[async_trait(?Send)]
impl Authentication for MockAuth {
    async fn get_token(&self, code: &str) -> Result<String> {
        match code {
            VALID_CODE => Ok(VALID_TOKEN.to_owned()),
            _ => Err(actix_web::error::ErrorNotFound("error")),
        }
    }
    async fn get_user_id(&self, token: &str) -> Result<GitHubUserResponse> {
        match token {
            VALID_TOKEN => Ok(GitHubUserResponse::default()),
            _ => Err(actix_web::error::ErrorNotFound("error")),
        }
    }
}

fn url(path: &str, port: u16) -> String {
    format!("http://localhost:{}{}", port, path)
}

async fn setup() -> u16 {
    utils::initialize_and_connect_to_test_sql().await;
    let mut rng = rand::thread_rng();
    rng.gen::<u16>() % 30000 + 30000
}

#[tokio::test]
async fn test_virtual_contest() {
    let port = setup().await;
    let server = actix_rt::spawn(async move {
        let pg_pool = sql_client::initialize_pool(utils::get_sql_url_from_env())
            .await
            .unwrap();
        run_server(pg_pool, MockAuth, port).await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    reqwest::get(url(
        &format!("/internal-api/authorize?code={}", VALID_CODE),
        port,
    ))
    .await
    .unwrap();
    let cookie_header = format!("token={}", VALID_TOKEN);

    let response = reqwest::Client::new()
        .post(url("/internal-api/user/update", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "atcoder_user_id": "atcoder_user1"
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let response = reqwest::Client::new()
        .post(url("/internal-api/contest/create", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "title": "contest title",
            "memo": "contest memo",
            "start_epoch_second": 1,
            "duration_second": 2,
            "penalty_second": 0,
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.json::<Value>().await.unwrap();
    let contest_id = body["contest_id"].as_str().unwrap();

    let response = reqwest::Client::new()
        .post(url("/internal-api/contest/update", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "id": format!("{}", contest_id),
            "title": "contest title",
            "memo": "contest memo",
            "start_epoch_second": 1,
            "duration_second": 2,
            "penalty_second": 300,
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    let response = reqwest::Client::new()
        .get(url("/internal-api/contest/my", port))
        .header("Cookie", cookie_header.as_str())
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(
        response,
        json!([
            {
                "owner_user_id": "0",
                "duration_second": 2,
                "start_epoch_second": 1,
                "memo": "contest memo",
                "title": "contest title",
                "id": format!("{}", contest_id),
                "mode": null,
                "is_public": true,
                "penalty_second": 300,
            }
        ])
    );

    let response = reqwest::Client::new()
        .get(url("/internal-api/contest/joined", port))
        .header("Cookie", cookie_header.as_str())
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(response, json!([]));

    let response = reqwest::Client::new()
        .post(url("/internal-api/contest/join", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "contest_id": format!("{}", contest_id),
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    let response = reqwest::Client::new()
        .get(url("/internal-api/contest/joined", port))
        .header("Cookie", cookie_header.as_str())
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(
        response,
        json!([
            {
                "owner_user_id": "0",
                "duration_second": 2,
                "start_epoch_second": 1,
                "memo": "contest memo",
                "title": "contest title",
                "id": format!("{}", contest_id),
                "mode": null,
                "is_public": true,
                "penalty_second": 300,
            }
        ])
    );

    let response = reqwest::Client::new()
        .post(url("/internal-api/contest/leave", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "contest_id": format!("{}", contest_id),
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    let response = reqwest::Client::new()
        .get(url("/internal-api/contest/joined", port))
        .header("Cookie", cookie_header.as_str())
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(response, json!([]));

    let response = reqwest::Client::new()
        .post(url("/internal-api/contest/join", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "contest_id": format!("{}", contest_id),
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    let response = reqwest::Client::new()
        .get(url("/internal-api/contest/joined", port))
        .header("Cookie", cookie_header.as_str())
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(
        response,
        json!([
            {
                "owner_user_id": "0",
                "duration_second": 2,
                "start_epoch_second": 1,
                "memo": "contest memo",
                "title": "contest title",
                "id": format!("{}", contest_id),
                "mode": null,
                "is_public": true,
                "penalty_second": 300,
            }
        ])
    );

    let response = reqwest::Client::new()
        .post(url("/internal-api/contest/item/update", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "contest_id": format!("{}", contest_id),
            "problems": [{ "id": "problem_1", "point": 100 }],
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    let response = reqwest::Client::new()
        .post(url("/internal-api/contest/item/update", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "contest_id": format!("{}", contest_id),
            "problems": [{ "id": "problem_1", "point": 100 }],
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    let response = reqwest::Client::new()
        .post(url("/internal-api/contest/item/update", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "contest_id": format!("{}", contest_id),
            "problems": [{ "id": "problem_1", "point": 100 }, { "id": "problem_2" }],
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    let response = reqwest::Client::new()
        .get(url("/internal-api/contest/joined", port))
        .header("Cookie", cookie_header.as_str())
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(
        response,
        json!([
            {
                "owner_user_id": "0",
                "duration_second": 2,
                "start_epoch_second": 1,
                "memo": "contest memo",
                "title": "contest title",
                "id": format!("{}", contest_id),
                "mode": null,
                "is_public": true,
                "penalty_second": 300,
            }
        ])
    );

    let response = reqwest::get(url(
        &format!("/internal-api/contest/get/{}", contest_id),
        port,
    ))
    .await
    .unwrap()
    .json::<Value>()
    .await
    .unwrap();
    assert_eq!(
        response,
        json!({
            "info": {
                "owner_user_id": "0",
                "duration_second": 2,
                "start_epoch_second": 1,
                "memo": "contest memo",
                "title": "contest title",
                "id": format!("{}", contest_id),
                "mode": null,
                "is_public": true,
                "penalty_second": 300,
            },
            "problems": [{ "id": "problem_1", "point": 100, "order": null }, { "id": "problem_2", "point": null, "order": null }],
            "participants": ["atcoder_user1"],
        })
    );

    let response = reqwest::get(url("/internal-api/contest/recent", port))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(
        response,
        json!([
            {
                "owner_user_id": "0",
                "duration_second": 2,
                "start_epoch_second": 1,
                "memo": "contest memo",
                "title": "contest title",
                "is_public": true,
                "id": format!("{}", contest_id),
                "mode": null,
                "penalty_second": 300,
            }
        ])
    );

    server.abort();
    server.await.unwrap_err();
}

#[tokio::test]
async fn test_virtual_contest_visibility() {
    let port = setup().await;
    let server = actix_rt::spawn(async move {
        let pg_pool = sql_client::initialize_pool(utils::get_sql_url_from_env())
            .await
            .unwrap();
        run_server(pg_pool, MockAuth, port).await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    reqwest::get(url(
        &format!("/internal-api/authorize?code={}", VALID_CODE),
        port,
    ))
    .await
    .unwrap();
    let cookie_header = format!("token={}", VALID_TOKEN);

    let response = reqwest::Client::new()
        .post(url("/internal-api/contest/create", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "title": "visible",
            "memo": "",
            "start_epoch_second": 1,
            "duration_second": 2,
            "penalty_second": 300,
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.json::<Value>().await.unwrap();
    let contest_id = body["contest_id"].as_str().unwrap();

    let response = reqwest::get(url("/internal-api/contest/recent", port))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(response[0]["id"].as_str().unwrap(), contest_id);
    assert_eq!(response.as_array().unwrap().len(), 1);

    let response = reqwest::Client::new()
        .post(url("/internal-api/contest/update", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "id": format!("{}", contest_id),
            "title": "invisible",
            "memo": "",
            "start_epoch_second": 1,
            "duration_second": 2,
            "is_public": false,
            "penalty_second": 300,
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    let response = reqwest::get(url("/internal-api/contest/recent", port))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(response.as_array().unwrap().len(), 0);

    let response = reqwest::Client::new()
        .post(url("/internal-api/contest/create", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "title": "invisible",
            "memo": "",
            "start_epoch_second": 1,
            "duration_second": 2,
            "is_public": false,
            "penalty_second": 300,
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let body = response.json::<Value>().await.unwrap();
    let contest_id = body["contest_id"].as_str().unwrap();

    let response = reqwest::get(url("/internal-api/contest/recent", port))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(response.as_array().unwrap().len(), 0);

    let response = reqwest::Client::new()
        .post(url("/internal-api/contest/update", port))
        .header("Cookie", cookie_header.as_str())
        .json(&json!({
            "id": contest_id,
            "title": "visible",
            "memo": "",
            "start_epoch_second": 1,
            "duration_second": 2,
            "is_public": true,
            "penalty_second": 300,
        }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    let response = reqwest::get(url("/internal-api/contest/recent", port))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"].as_str().unwrap(), contest_id);

    server.abort();
    server.await.unwrap_err();
}
