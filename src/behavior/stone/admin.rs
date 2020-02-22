use crate::behavior::{StoneBehavior, ThreadResult};
use crate::vkapi::{Client, VkApi, VkMessage, VkMessagesApi, VkUser, VkUsersApi};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref ADMIN_ACT: Mutex<HashMap<i64, AdminAct>> = Mutex::new(HashMap::new());
}

enum AdminAct {
    None,
    EditUser(VkUser),
}

pub trait StoneAdmin<C: Client> {
    fn reply_admin<'s>(&'s self, vk: &VkApi<C>, msg: &VkMessage) -> ThreadResult<'s>;
}

const USAGE_START: &str = "Отправь ссылку на страницу пользователя в формате vk.com/name";
fn usage_no_user(name: &str) -> String {
    format!("Пользователь {} не найден. {}", name, USAGE_START)
}
fn usage_user(user: &VkUser) -> String {
    format!(
        r#"{}
Напиши "этап n", чтобы перевести пользователя на другой этап (например, "этап 2").
Напиши "отмена", чтобы выбрать другого пользователя.
"#,
        user
    )
}

impl<C: Client> StoneAdmin<C> for StoneBehavior {
    fn reply_admin<'s>(&'s self, vk: &VkApi<C>, msg: &VkMessage) -> ThreadResult<'s> {
        let mut act = ADMIN_ACT.lock().unwrap();
        let next_act = match act.remove(&msg.from_id).unwrap_or(AdminAct::None) {
            AdminAct::None => {
                if msg.text.starts_with("vk.com/") {
                    let name = msg.text[7..].trim_end_matches('/');
                    if let Some(user) = vk.get_user(name)? {
                        vk.send(msg.from_id, &usage_user(&user), None)?;
                        AdminAct::EditUser(user)
                    } else {
                        vk.send(msg.from_id, &usage_no_user(name), None)?;
                        AdminAct::None
                    }
                } else {
                    vk.send(msg.from_id, USAGE_START, None)?;
                    AdminAct::None
                }
            }
            AdminAct::EditUser(user) => {
                let command = msg.text.trim().to_lowercase();
                match command.as_str() {
                    "отмена" => {
                        vk.send(msg.from_id, USAGE_START, None)?;
                        AdminAct::None
                    }
                    _ if command.starts_with("этап ") => {
                        use crate::behavior::stone::consts::{STAGE_HASHES, STORAGE_STAGE_HASH};
                        match u64::from_str_radix(&command.replace("этап ", ""), 10) {
                            Ok(st) if st > 0 && st as usize <= STAGE_HASHES.len() => {
                                self.storage.hash_set(STORAGE_STAGE_HASH, user.id, st - 1)?;
                                let reply = format!("{} теперь на этапе {}", user, st);
                                vk.send(msg.from_id, &reply, None)?;
                                AdminAct::None
                            }
                            _ => {
                                let reply = "Пришли номер этапа как число, например, \"этап 2\"";
                                vk.send(msg.from_id, reply, None)?;
                                AdminAct::EditUser(user)
                            }
                        }
                    }
                    _ => {
                        vk.send(msg.from_id, &usage_user(&user), None)?;
                        AdminAct::EditUser(user)
                    }
                }
            }
        };
        act.insert(msg.from_id, next_act);
        Ok(())
    }
}
