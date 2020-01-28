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

    for photo in msg.attachments {
        let img_bytes = vk.fetch_photo(&photo).await?;
        let img = image::load_from_memory(&img_bytes)?;

        let hasher = img_hash::HasherConfig::new()
            .hash_alg(img_hash::HashAlg::Mean)
            .preproc_dct()
            .to_hasher();
        let hash = hasher.hash_image(&img);

        println!("Image hash: {:?}", hash);
    }
    Ok(())
}
