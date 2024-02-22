use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Response {
    #[serde(rename = "accessToken")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,

    #[serde(rename = "clientToken")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_token: Option<String>,

    #[serde(rename = "availableProfiles")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_profiles: Option<Vec<AvailableProfiles>>,

    #[serde(rename = "user")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,

    #[serde(rename = "selectedProfile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_profile: Option<AvailableProfiles>,

    #[serde(rename = "error")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    #[serde(rename = "errorMessage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AvailableProfiles {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "properties")]
    pub properties: Vec<u8>,
}

pub async fn send(
    url: String,
    access_token: String,
    client_token: String,
    request_user: bool,
    select_name: String,
    select_id: String,
) -> Result<Response, String> {
    let client: Client = Client::new();
    let url = format!("https://{}/api/yggdrasil/authserver/refresh", url);
    match client
        .post(url)
        .json(&json!({
            "accessToken": access_token,
            "clientToken": client_token,
            "requestUser": request_user,
            "selectedProfile": {
                "id": select_id,
                "name": select_name
            }
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
            "xxx".to_string(),
            "xxx".to_string(),
            true,
            "xxx".to_string(),
            "xxx".to_string(),
        )
        .await;
        match response {
            Ok(response) => {
                assert_eq!(response.access_token, None);
                assert_eq!(response.client_token, None);
                assert_eq!(response.available_profiles, None);
                assert_eq!(response.user, None);
                assert_eq!(response.selected_profile, None);
                assert_eq!(
                    response.error,
                    Some("ForbiddenOperationException".to_string())
                );
                assert_eq!(
                    response.error_message,
                    Some("无效的 AccessToken，请重新登录".to_string())
                );
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}
