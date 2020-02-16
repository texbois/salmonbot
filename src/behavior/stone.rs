use crate::behavior::{Behavior, ThreadResult};
use crate::img_match::ImageMatcher;
use crate::storage::Storage;
use crate::vkapi::{Client, VkApi, VkMessage, VkMessagesApi, VkPhotosApi};
use crate::MSG_DELAY;

#[rustfmt::skip]
const STAGE_HASHES: [&[(&str, [u8; 14])]; 1] = [
    // stage one
    &[
        ("уа", [188, 202, 74, 57, 105, 113, 196, 54, 203, 51, 153, 77, 101, 234]),
        ("п", [156, 230, 168, 118, 83, 198, 98, 20, 212, 250, 221, 237, 60, 186]),
        ("ч", [88, 37, 253, 54, 89, 185, 153, 48, 109, 229, 212, 107, 52, 114]),
        ("о", [44, 211, 33, 245, 176, 110, 77, 92, 211, 111, 171, 92, 149, 122])
    ],
    // stages two+: tbd
];

const STAGE_COMPLETION_PICS: [(&[u8], &str); 1] =
    [(include_bytes!("../../static/stone_stage_1.jpg"), "jpg")];

const STORAGE_STAGE_HASH: &str = "stone_stage";
const STORAGE_LETTER_BUCKET_PREF: &str = "stone_letter_";

pub struct StoneBehavior {
    matcher: ImageMatcher,
    storage: Storage,
}

impl StoneBehavior {
    pub fn new(storage: Storage) -> Self {
        let matcher = ImageMatcher::new();
        Self { matcher, storage }
    }
}

impl std::fmt::Display for StoneBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stone")
    }
}

impl<C: Client> Behavior<C> for StoneBehavior {
    fn process_on_own_thread<'s>(&'s self, vk: &VkApi<C>, msg: &VkMessage) -> ThreadResult<'s> {
        // hincrby 0 is analogous to get or set to 0
        let player_stage = self.storage.hash_incr(STORAGE_STAGE_HASH, msg.from_id, 0)?;
        if player_stage == STAGE_HASHES.len() as i64 {
            return Ok(());
        }

        let buckets_should_match = STAGE_HASHES[player_stage as usize]
            .iter()
            .map(|(letter, _)| [STORAGE_LETTER_BUCKET_PREF, letter].concat())
            .collect::<Vec<_>>();
        let mut buckets_matched: Vec<String> = Vec::new();

        for att in msg.all_attachments() {
            let image = vk.download_photo(att)?;
            let hash = self.matcher.hash(&image)?;

            for (stage, letter_hashes) in STAGE_HASHES.iter().enumerate() {
                for (letter, letter_hash) in letter_hashes.iter() {
                    if ImageMatcher::matches(letter_hash, &hash) {
                        if player_stage == stage as i64 {
                            buckets_matched.push([STORAGE_LETTER_BUCKET_PREF, letter].concat());
                        } else {
                            std::thread::sleep(MSG_DELAY);
                            return vk.send(msg.from_id, wrong_stage_text(player_stage), None);
                        }
                    }
                }
            }
        }
        let total_matched = self.storage.sets_add_and_count_containing(
            &buckets_matched,
            &buckets_should_match,
            msg.from_id,
        )?;
        if total_matched == buckets_should_match.len() {
            let _next_stage = self.storage.hash_incr(STORAGE_STAGE_HASH, msg.from_id, 1)?;
            let photo =
                vk.upload_message_photo(msg.from_id, STAGE_COMPLETION_PICS[player_stage as usize])?;

            std::thread::sleep(MSG_DELAY);
            vk.send(msg.from_id, "", Some(&photo))
        } else {
            let reply = format!("{}/{}", total_matched, buckets_should_match.len());
            vk.send(msg.from_id, &reply, None)
        }
    }
}

fn wrong_stage_text(stage: i64) -> &'static str {
    match stage {
        0 => "Нужно собрать первое заклинание",
        1 => "Нужно собрать второе заклинание",
        2 => "Нужно собрать третье заклинание",
        3 => "Нужно собрать последнее заклинание",
        _ => "",
    }
}
