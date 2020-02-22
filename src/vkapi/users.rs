use crate::vkapi::{Client, VkApi};
use crate::BotResult;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct VkUser {
    pub id: i64,
    pub screen_name: String,
    pub first_name: String,
    pub last_name: String,
}

impl std::fmt::Display for VkUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} (@{}, id {})",
            self.first_name, self.last_name, self.screen_name, self.id
        )
    }
}

pub trait VkUsersApi {
    fn get_user(&self, screen_name: &str) -> BotResult<Option<VkUser>>;
}

impl<C: Client> VkUsersApi for VkApi<C> {
    fn get_user(&self, screen_name: &str) -> BotResult<Option<VkUser>> {
        let mut response: serde_json::Value = self.call_api(
            "users.get",
            &[("user_ids", screen_name), ("fields", "screen_name")],
            None,
        )?;
        if let Some(users) = response.get_mut("response").and_then(|r| r.as_array_mut()) {
            if users.len() == 1 {
                return serde_json::from_value(users.remove(0)).map_err(|e| e.into());
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user() {
        let vk = VkApi {
            client: crate::vkapi::http::TestClient::new("get_user.json"),
            token: "token".into(),
            community_name: "sample_community".into(),
            community_id: "1001".into(),
        };
        let user = vk.get_user("michiganjfrog").unwrap().unwrap();
        assert_eq!(user.screen_name, "michiganjfrog");
        assert_eq!(user.id, 1);
        assert_eq!(user.first_name, "Hello");
        assert_eq!(user.last_name, "My Baby");
        assert_eq!(format!("{}", user), "Hello My Baby (@michiganjfrog, id 1)");
    }

    #[test]
    fn test_get_user_not_found() {
        let vk = VkApi {
            client: crate::vkapi::http::TestClient::new("get_user_missing.json"),
            token: "token".into(),
            community_name: "sample_community".into(),
            community_id: "1001".into(),
        };
        let user = vk.get_user("wednesdayfrog").unwrap();
        assert!(user.is_none());
    }
}
