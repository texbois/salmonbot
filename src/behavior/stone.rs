use crate::behavior::{Behavior, ThreadResult};
use crate::img_match::ImageMatcher;
use crate::storage::Storage;
use crate::vkapi::{Client, VkApi, VkMessage, VkMessagesApi, VkPhotosApi};
use crate::MSG_DELAY_FAIL;
use crate::MSG_DELAY_SUCCESS;

#[rustfmt::skip]
pub const STAGE_HASHES: [&[(&str, [u8; 18])]; 2] = [
    // stage one
    &[
        ("1-уа", [188, 149, 171, 74, 147, 173, 156, 226, 76, 182, 22, 79, 73, 153, 169, 153, 245, 36]),
        ("1-п", [156, 205, 163, 181, 183, 74, 177, 177, 182, 148, 40, 235, 239, 157, 157, 143, 221, 227]),
        ("1-ч", [88, 74, 244, 183, 147, 43, 110, 76, 215, 48, 90, 149, 167, 61, 141, 141, 57, 231]),
        ("1-о", [172, 134, 151, 169, 143, 214, 91, 162, 73, 92, 166, 63, 91, 202, 171, 37, 181, 214])
    ],
    // stage two
    &[
        ("2-ма-м", [100, 150, 82, 226, 171, 189, 85, 202, 168, 212, 150, 107, 37, 73, 91, 140, 166, 236]),
        ("2-ма-а", [100, 150, 82, 226, 171, 189, 85, 202, 168, 212, 150, 107, 37, 73, 91, 140, 166, 236]),
        ("2-м", [74, 146, 177, 165, 124, 108, 220, 77, 148, 196, 102, 184, 182, 84, 232, 137, 24, 179]),
        ("2-э", [108, 178, 91, 108, 183, 92, 179, 140, 51, 84, 150, 169, 45, 82, 75, 237, 150, 42]),
        ("2-о", [50, 48, 212, 151, 44, 75, 204, 221, 182, 170, 41, 212, 230, 94, 118, 204, 91, 99])
    ]
    // stages three+: tbd
];

const STAGE_COMPLETION_PICS: [(&[u8], &str); 2] = [
    (include_bytes!("../../static/stone_stage_1.jpg"), "jpg"),
    (include_bytes!("../../static/stone_stage_2.jpg"), "jpg"),
];

const STORAGE_STAGE_HASH: &str = "stone_stage";
pub fn storage_letter_bucket(letter: &str) -> String {
    ["stone_letter_", letter].concat()
}

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
            .map(|(letter, _)| storage_letter_bucket(letter))
            .collect::<Vec<_>>();
        let mut buckets_matched: Vec<String> = Vec::new();

        for att in msg.all_attachments() {
            let image = vk.download_photo(att)?;
            let hash = self.matcher.hash(&image)?;

            for (stage, letter_hashes) in STAGE_HASHES.iter().enumerate() {
                for (letter, letter_hash) in letter_hashes.iter() {
                    if ImageMatcher::matches(letter_hash, &hash) {
                        if player_stage == stage as i64 {
                            buckets_matched.push(storage_letter_bucket(letter));
                        } else {
                            std::thread::sleep(MSG_DELAY_FAIL);
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
            std::thread::sleep(MSG_DELAY_SUCCESS);

            let completion_pic = STAGE_COMPLETION_PICS[player_stage as usize];
            let photo = vk.upload_message_photo(msg.from_id, completion_pic)?;
            vk.send(msg.from_id, "", Some(&photo))?;

            let _ = self.storage.hash_incr(STORAGE_STAGE_HASH, msg.from_id, 1)?;
        } else {
            let reply = format!("{}/{}", total_matched, buckets_should_match.len());

            std::thread::sleep(MSG_DELAY_SUCCESS);
            vk.send(msg.from_id, &reply, None)?;
        }
        Ok(())
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
