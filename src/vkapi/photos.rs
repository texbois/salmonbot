use crate::vkapi::{Client, VkApi, VkPhoto};
use crate::BotResult;
use serde_derive::Deserialize;

pub trait VkPhotosApi {
    fn download_photo(&self, photo: &VkPhoto) -> BotResult<Vec<u8>>;
    fn upload_message_photo(&self, peer_id: i64, photo: (&[u8], &str)) -> BotResult<String>;
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

    fn upload_message_photo(&self, peer_id: i64, photo: (&[u8], &str)) -> BotResult<String> {
        let mut retries = 0;
        let upload = loop {
            let resp = get_upload_url(&self, peer_id).and_then(|url| {
                self.client
                    .post_multipart::<VkPhotoUploadResponse>(&url, "photo", photo, None)
            });
            match resp {
                Err(e) if retries == 0 => {
                    retries += 1;
                    eprintln!(
                        "Error when uploading photo to {}'s messages: {}. Retrying once",
                        peer_id, e
                    );
                }
                _ => break resp,
            }
        }?;

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
        Err(format!("Unable to upload photo, unexpected response: {:?}", media).into())
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
            .upload_message_photo(
                101,
                (include_bytes!("../../tests/fixtures/test.jpg"), "jpg"),
            )
            .unwrap();
        assert_eq!(media_obj, "photo101_7777777");
    }
}
