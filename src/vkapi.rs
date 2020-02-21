mod http;
mod long_poll;
mod messages;
mod photos;
mod types;
mod users;
pub use http::Client;
pub use long_poll::{VkLongPoll, VkLongPollState};
pub use messages::VkMessagesApi;
pub use photos::VkPhotosApi;
pub use types::{VkMessage, VkPhoto};
pub use users::VkUsersApi;

pub struct VkApi<C: Client> {
    pub client: C,
    token: String,
    community_id: String,
    community_name: String,
}

impl<C: Client> std::fmt::Display for VkApi<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Community \"{}\" (id {})",
            self.community_name, self.community_id
        )
    }
}

#[inline]
fn api_request<'a>(
    method: &str,
    query: &[(&'a str, &'a str)],
    token: &'a str,
) -> (String, Vec<(&'a str, &'a str)>) {
    let url = ["https://api.vk.com/method/", method].concat();
    let api_query = [query, &[("v", "5.103"), ("access_token", token)]].concat();
    (url, api_query)
}

impl<C: Client> VkApi<C> {
    pub fn new(client: C, token: String) -> crate::BotResult<Self> {
        let (comm_url, comm_query) = api_request("groups.getById", &[], &token);
        let communities: serde_json::Value =
            client.get_json(&comm_url, &comm_query, Some("response"))?;

        let mut community = match communities {
            serde_json::Value::Array(mut comms) if comms.len() == 1 => comms.remove(0),
            resp => {
                return Err(format!(
                    "The bot should be linked is linked to none or multiple communities, got: {:?}",
                    resp
                )
                .into())
            }
        };
        let community_id = match community.get("id").and_then(|id| id.as_u64()) {
            Some(id) => id.to_string(),
            _ => return Err(format!("Group item missing \"id\": {:?}", community).into()),
        };
        let community_name = match community.get_mut("name").map(|n| n.take()) {
            Some(serde_json::Value::String(name)) => name,
            _ => return Err(format!("Group item missing \"name\" {:?}", community).into()),
        };

        Ok(Self {
            client,
            token,
            community_id,
            community_name,
        })
    }

    fn call_api<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        query: &[(&str, &str)],
        json_response_key: Option<&str>,
    ) -> crate::BotResult<T> {
        let (url, api_query) = api_request(method, query, &self.token);
        self.client.get_json(&url, &api_query, json_response_key)
    }
}
