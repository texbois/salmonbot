use crate::vkapi::{Client, VkApi, VkPhoto};
use crate::BotResult;
use serde_derive::Deserialize;
use std::path::Path;

pub trait VkPhotosApi {
    fn download_photo(&self, photo: &VkPhoto) -> BotResult<Vec<u8>>;
    fn upload_message_photo<P: AsRef<Path>>(&self, peer_id: i64, path: P) -> BotResult<String>;
}

#[derive(Debug, Deserialize)]
pub struct VkPhotoUploadResponse {
    server: i64,
    photo: String,
    hash: String,
}

impl<C: Client> VkPhotosApi for VkApi<C> {
    fn download_photo(&self, photo: &VkPhoto) -> BotResult<Vec<u8>> {
        self.client.fetch(&photo.0, &[], &[], None)
    }

    fn upload_message_photo<P: AsRef<Path>>(&self, peer_id: i64, path: P) -> BotResult<String> {
        let image_ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg");
        let image = std::fs::read(&path)?;
        let upload_url = get_upload_url(&self, peer_id)?;
        let upload: VkPhotoUploadResponse =
            self.client
                .post_multipart(&upload_url, "photo", &image, image_ext, None)?;
        let media: serde_json::Value = self.call_api(
            "photos.saveMessagesPhoto",
            &[
                ("server", &upload.server.to_string()),
                ("photo", &upload.photo),
                ("hash", &upload.hash),
            ],
            Some("response"),
        )?;
        if let Some(m) = media.as_array() {
            if m.len() == 1 {
                if let Some(id) = m[0].get("id").and_then(|i| i.as_i64()) {
                    if let Some(owner) = m[0].get("owner_id").and_then(|i| i.as_i64()) {
                        return Ok(format!("photo{}_{}", owner, id));
                    }
                }
            }
        }
        Err(format!(
            "Unable to upload {}, unexpected media response: {:?}",
            path.as_ref().display(),
            media
        )
        .into())
    }
}

fn get_upload_url<C: Client>(vk: &VkApi<C>, peer_id: i64) -> BotResult<String> {
    let mut server: serde_json::Value = vk.call_api(
        "photos.getMessagesUploadServer",
        &[("peer_id", &peer_id.to_string())],
        Some("response"),
    )?;
    if let Some(serde_json::Value::String(url)) = server.get_mut("upload_url").map(|u| u.take()) {
        Ok(url)
    } else {
        Err(format!(
            "Unexpected photos.getMessagesUploadServer response: {}",
            server,
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vkapi::photos::VkPhotosApi;

    #[test]
    fn test_message_upload() {
        let vk = VkApi {
            client: crate::vkapi::http::TestClient::new("photo_message_upload.json"),
            token: "token".into(),
            community_name: "sample_community".into(),
            community_id: "1001".into(),
        };
        let media_obj = vk
            .upload_message_photo(101, "tests/fixtures/test.jpg")
            .unwrap();
        assert_eq!(media_obj, "photo101_7777777");
    }
}
