use crate::behavior::{Behavior, ThreadResult};
use crate::storage::Storage;
use crate::vkapi::{Client, VkApi, VkMessage, VkMessagesApi};

pub struct StatsBehavior {
    storage: Storage,
    admin_ids: Vec<i64>,
}

impl StatsBehavior {
    pub fn new(storage: Storage) -> Self {
        let admin_ids = std::env::var("STATS_ADMIN_IDS")
            .expect("Provide admin user ids via the STATS_ADMIN_IDS environment variable")
            .split(',')
            .map(|id| i64::from_str_radix(id, 10).unwrap())
            .collect();

        Self { storage, admin_ids }
    }
}

impl std::fmt::Display for StatsBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stats")
    }
}

impl<C: Client> Behavior<C> for StatsBehavior {
    fn process_on_own_thread<'s>(&'s self, vk: &VkApi<C>, msg: &VkMessage) -> ThreadResult<'s> {
        if !self.admin_ids.contains(&msg.from_id) {
            return Ok(())
        }

        use std::fmt::Write;
        let mut s = String::new();

        s.push_str("Камень в лесу:\n");

        use crate::behavior::stone::{storage_letter_bucket, STAGE_HASHES};
        for (stage, letters_hashes) in STAGE_HASHES.iter().enumerate() {
            let letters = letters_hashes
                .into_iter()
                .map(|&(letter, _)| letter)
                .collect::<Vec<_>>();
            let letter_completions = self
                .storage
                .sets_len(letters.iter().map(|&l| storage_letter_bucket(l)))?;

            write!(&mut s, "Этап {}:\n", stage + 1).unwrap();
            for (letter, completed_by) in letters.iter().zip(letter_completions) {
                write!(&mut s, "- {}: {}\n", letter, completed_by).unwrap();
            }
        }

        use crate::behavior::chest::STORAGE_COMPL_SET as CHEST_KEY;
        let chest_completions = self.storage.sets_len([CHEST_KEY].iter())?[0];
        write!(&mut s, "\nСундук: {}", chest_completions).unwrap();

        vk.send(msg.from_id, &s, None)
    }
}
