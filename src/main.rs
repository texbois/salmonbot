#![recursion_limit = "256"]

mod vkapi;
use vkapi::{Client, VkApi, VkMessage};

pub type BotResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() {
    let token = std::env::var("COMMUNITY_TOKEN")
        .expect("Provide a valid API token via the COMMUNITY_TOKEN environment variable");

    match run_bot(token) {
        Ok(_) => (),
        Err(err) => eprintln!("Error: {}", err),
    }
}

fn run_bot(token: String) -> BotResult<()> {
    let client = ureq::agent();
    let vk = VkApi::new(client, token)?;
    println!("Running {}", vk);

    let mut lp = vk.init_long_poll()?;
    loop {
        lp.poll_once(|u| process_message(&vk, u))?;
    }
}

fn process_message<C: Client>(vk: &VkApi<C>, msg: VkMessage) -> BotResult<()> {
    println!("Message: {:?}", msg);

    const HASH_FOOD: [u8; 14] = [50, 43, 61, 197, 89, 22, 36, 42, 27, 149, 196, 74, 50, 183];
    const HASH_WRENCH: [u8; 14] = [
        220, 149, 201, 150, 157, 70, 121, 74, 100, 98, 218, 101, 142, 77,
    ];
    const HASH_BIRD: [u8; 14] = [208, 92, 39, 121, 50, 47, 89, 88, 18, 77, 107, 18, 109, 45];

    let attachments = msg.all_attachments();
    if attachments.len() == 0 {
        vk.send_message(msg.from_id, "Я тебя не вижу!")
    } else {
        vk.send_message(msg.from_id, "Дай подумать...")?;

        let hasher = img_hash::HasherConfig::new()
            .hash_alg(img_hash::HashAlg::DoubleGradient)
            .hash_size(14, 14)
            .preproc_dct()
            .to_hasher();

        let mut results: Vec<String> = Vec::new();
        for photo in attachments {
            let image = image::load_from_memory(&vk.fetch_photo(photo)?)?;
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
        vk.send_message(msg.from_id, &results.join(" "))
    }
}
