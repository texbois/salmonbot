use serde_json;

#[derive(Debug)]
pub struct VkApi {
    token: String,
    community_id: u64,
    community_name: String,
    client: reqwest::Client,
}

impl VkApi {
    pub async fn new(token: String) -> crate::BotResult<Self> {
        let client = reqwest::Client::new();

        let mut communities =
            call_api::<serde_json::Value>(&client, &token, "groups.getById", &[]).await?;

        let mut community = match communities["response"].take() {
            serde_json::Value::Array(comms) if comms.len() == 0 => {
                return Err("The bot is not linked to any communities".into())
            }
            serde_json::Value::Array(mut comms) => comms.remove(0),
            resp => {
                return Err(
                    format!("Unexpected \"response\" for groups.getById: {:?}", resp).into(),
                )
            }
        };

        let community_id = community["id"].as_u64().unwrap();
        let community_name = match community["name"].take() {
            serde_json::Value::String(name) => name,
            _ => return Err("Missing community name in groups.getById response".into()),
        };

        Ok(Self {
            token,
            community_id,
            community_name,
            client,
        })
    }
}

#[inline]
async fn call_api<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    token: &str,
    method: &str,
    query: &[(&str, &str)],
) -> crate::BotResult<T> {
    client
        .get(&format!("https://api.vk.com/method/{}", method))
        .query(query)
        .query(&[("v", "5.103"), ("access_token", token)])
        .send()
        .await?
        .json::<T>()
        .await
        .map_err(|e| e.into())
}
