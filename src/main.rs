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

    const HASH_FOOD: [u8; 14] = [50, 43, 61, 197, 89, 22, 36, 42, 27, 149, 196, 74, 50, 183];
    const HASH_WRENCH: [u8; 14] = [
        220, 149, 201, 150, 157, 70, 121, 74, 100, 98, 218, 101, 142, 77,
    ];
    const HASH_BIRD: [u8; 14] = [208, 92, 39, 121, 50, 47, 89, 88, 18, 77, 107, 18, 109, 45];

    let attachments = msg
        .attachments
        .iter()
        .chain(msg.forwarded.iter().flat_map(|m| &m.attachments))
        .collect::<Vec<_>>();
    if attachments.len() == 0 {
        vk.send_message(msg.from_id, "Я тебя не вижу!").await
    } else {
        vk.send_message(msg.from_id, "Дай подумать...").await?;

        let hasher = img_hash::HasherConfig::new()
            .hash_alg(img_hash::HashAlg::DoubleGradient)
            .hash_size(14, 14)
            .preproc_dct()
            .to_hasher();

        let mut results: Vec<String> = Vec::new();
        for photo in attachments {
            let image = image::load_from_memory(&vk.fetch_photo(photo).await?)?;
            let hash = hasher.hash_image(&image);

            let dist_food = hamming::distance(&HASH_FOOD, hash.as_bytes());
            let dist_wrench = hamming::distance(&HASH_WRENCH, hash.as_bytes());
            let dist_bird = hamming::distance(&HASH_BIRD, hash.as_bytes());

            results.push(if dist_food <= 2 {
                format!("Еда {}!", dist_food)
            } else if dist_wrench <= 2 {
                format!("Гаечный ключ {}!", dist_wrench)
            } else if dist_bird <= 2 {
                format!("Мудрая Птица {}!", dist_bird)
            } else {
                format!(
                    "Такого не знаю... (еда: {}, ключ: {}, птица: {}, h: {:?})",
                    dist_food,
                    dist_wrench,
                    dist_bird,
                    hash.as_bytes()
                )
            });
        }
        vk.send_message(msg.from_id, &results.join(" ")).await
    }
}
