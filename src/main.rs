use aws_config::BehaviorVersion;
use aws_sdk_sts::types::{Credentials, PolicyDescriptorType};
use dotenvy::dotenv;
use reqwest::Url;
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    {
        dotenv().expect(".env file not found");
    }
    let aws_profile_name = env::var("AWS_PROFILE").expect("AWS_PROFILE not set");
    let aws_config = aws_config::defaults(BehaviorVersion::v2025_01_17())
        .profile_name(aws_profile_name)
        .load()
        .await;

    let aws_sdk_sts_client = aws_sdk_sts::Client::new(&aws_config);

    let policy_arn = PolicyDescriptorType::builder()
        .set_arn(Some(
            "arn:aws:iam::aws:policy/AdministratorAccess".to_string(),
        ))
        .build();

    let data = aws_sdk_sts_client
        .get_federation_token()
        .set_name(Some("imlikett".to_string()))
        .set_policy_arns(Some(vec![policy_arn]))
        .set_duration_seconds(Some(43200))
        .send()
        .await
        .unwrap();

    let Credentials {
        access_key_id,
        secret_access_key,
        session_token,
        ..
    } = data.credentials().unwrap();

    let credentials_json = json!({
        "sessionId": access_key_id,
        "sessionKey": secret_access_key,
        "sessionToken": session_token
    });

    let mut url = Url::parse("https://signin.aws.amazon.com/federation").unwrap();
    url.query_pairs_mut()
        .append_pair("Action", "getSigninToken")
        .append_pair("SessionDuration", "43200")
        .append_pair("Session", &credentials_json.to_string());
    let body = reqwest::get(url.clone())
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let data: serde_json::Value = serde_json::from_str(&body).unwrap();

    let mut url = Url::parse("https://signin.aws.amazon.com/federation").unwrap();
    url.query_pairs_mut()
        .append_pair("Action", "login")
        .append_pair("Issuer", "Example.org")
        .append_pair("Destination", "https://console.aws.amazon.com/")
        .append_pair("SigninToken", &data["SigninToken"].to_string());
    println!("url: {}", url);
}
