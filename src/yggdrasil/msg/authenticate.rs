use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Response {
    #[serde(rename = "accessToken")]
    #[serde(skip_serializing_if = "Option::is_none")]
    access_token: Option<String>,

    #[serde(rename = "clientToken")]
    #[serde(skip_serializing_if = "Option::is_none")]
    client_token: Option<String>,

    #[serde(rename = "availableProfiles")]
    #[serde(skip_serializing_if = "Option::is_none")]
    available_profiles: Option<Vec<AvailableProfiles>>,

    #[serde(rename = "user")]
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<User>,

    #[serde(rename = "error")]
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,

    #[serde(rename = "errorMessage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AvailableProfiles {
    #[serde(rename = "id")]
    id: String,

    #[serde(rename = "name")]
    name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
    #[serde(rename = "id")]
    id: String,

    #[serde(rename = "properties")]
    properties: Vec<u8>,
}
pub async fn send(
    url: String,
    username: String,
    password: String,
    request_user: bool,
) -> Result<Response, String> {
    let client: Client = Client::new();
    let url = format!("https://{}/api/yggdrasil/authserver/authenticate", url);
    match client
        .post(url)
        .json(&json!({
            "agent": {
                "name": "Minecraft",
                "version": 1
            },
            "username": username,
            "password": password,
            "requestUser": request_user
        }))
        .send()
        .await
    {
        Ok(response) => Ok(match response.json::<Response>().await {
            Ok(response) => response,
            Err(e) => return Err(e.to_string()),
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_send() {
        let response = send(
            "littleskin.cn".to_string(),
            "weizixiang0@outlook.com".to_string(),
            "xxx".to_string(),
            true,
        )
        .await;
        match response {
            Ok(response) => {
                assert_eq!(response.access_token, None);
                assert_eq!(response.client_token, None);
                assert_eq!(response.available_profiles, None);
                assert_eq!(response.user, None);
                assert_eq!(
                    response.error,
                    Some("ForbiddenOperationException".to_string())
                );
                assert_eq!(
                    response.error_message,
                    Some("输入的邮箱与密码不匹配".to_string())
                );
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}
