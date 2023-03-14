use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

use tokio::sync::{MutexGuard, RwLock};

use crate::ps_server_lib::custom_string_hash;

#[derive(Debug, Clone)]
pub struct TopicV2 {
    pub(crate) id: u64,
    pub(crate) sub_topics: Vec<Arc<TopicV2>>,
    pub(crate) name: String,
}

impl Hash for TopicV2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for TopicV2 {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TopicV2 {}

impl TopicV2 {
    pub fn new(id: u64, name: String) -> TopicV2 {
        TopicV2 {
            id,
            sub_topics: Vec::new(),
            name: String::from(name),
        }
    }

    pub fn add_sub_topic(&mut self, sub_topic: TopicV2) {
        self.sub_topics.push(Arc::from(sub_topic));
    }

    pub async fn create_topicsGPT(path: &str, mut root: Arc<RwLock<TopicV2>>) -> Result<u64, String> {
        let mut last_created_topic_id = { root.read().await.id };
        if path.len() == 0 || path.chars().nth(0).unwrap() != '/' {
            return Err("Invalid Path".to_string());
        }
        let mut topic_names: Vec<&str> = path.split("/").collect();

        topic_names.remove(0);

        if topic_names.is_empty() {
            return Err("c vid :c".to_string());
        }

        let mut topic_hash = String::from("");

        let mut current_topic = root;
        for topic_name in topic_names {
            topic_hash = topic_hash + "/" + topic_name;
            if topic_name.is_empty() {
                return Err("il s'est passé une dingeureie (topic_name is empty)".to_string());
            }
            let topic_id = custom_string_hash(&topic_hash);
            //     {
            //     let mut hasher = std::collections::hash_map::DefaultHasher::new();
            //     topic_hash.hash(&mut hasher);
            //     hasher.finish()
            // };

            let new_current_topic = {
                let mut writeTopic = current_topic.write().await;

                let existing_topic_idx = writeTopic.sub_topics.iter().position(|t| t.id == topic_id);
                if let Some(idx) = existing_topic_idx {
                    Arc::new(RwLock::new(writeTopic.sub_topics[idx].as_ref().to_owned()))
                } else {
                    let new_topic = TopicV2::new(topic_id, topic_name.to_string());
                    writeTopic.sub_topics.push(Arc::from(new_topic));
                    let new_topic_idx = writeTopic.sub_topics.len() - 1;
                    Arc::new(RwLock::new(writeTopic.sub_topics[new_topic_idx].as_ref().to_owned()))
                }
            };
            current_topic = new_current_topic;
            last_created_topic_id = topic_id;
        }
        Ok(last_created_topic_id)
    }
}
