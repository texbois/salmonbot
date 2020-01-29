#![recursion_limit = "256"]

mod vkapi;

pub type BotResult<T> = Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> BotResult<()> {
    let token = std::env::var("COMMUNITY_TOKEN")
        .expect("Provide a valid API token via the COMMUNITY_TOKEN environment variable");

    let vk = vkapi::VkApi::new(token).await?;
    vk.init_long_poll().await?.poll(process_message).await?;

    Ok(())
}

async fn process_message(vk: &vkapi::VkApi, msg: vkapi::VkMessage) -> BotResult<()> {
    println!("Message: {:?}", msg);

    if msg.attachments.len() == 2 {
        let image1 = image::load_from_memory(&vk.fetch_photo(&msg.attachments[0]).await?)?;
        let image2 = image::load_from_memory(&vk.fetch_photo(&msg.attachments[1]).await?)?;

        let hasher = img_hash::HasherConfig::new()
            .hash_alg(img_hash::HashAlg::Mean)
            .preproc_dct()
            .to_hasher();

        let hash1 = hasher.hash_image(&image1);
        let hash2 = hasher.hash_image(&image2);

        let distance = hamming::distance(hash1.as_bytes(), hash2.as_bytes());

        let reply = format!(
            "hash 1: {:?}, hash 2: {:?}, hamming distance: {}",
            hash1.as_bytes(),
            hash2.as_bytes(),
            distance
        );

        vk.send_message(msg.from_id, &reply).await
    } else {
        vk.send_message(msg.from_id, "Two attachments required for hash comparison")
            .await
    }
}
