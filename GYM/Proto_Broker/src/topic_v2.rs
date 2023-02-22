use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// il est 3h du mat je vais tuer qqn
// nsm les topics on verra demain

#[derive(Debug, Clone)]
pub struct TopicV2 {
    pub(crate) id: u64,
    pub(crate) sub_topics: Vec<TopicV2>,
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
    pub fn new(id: u64) -> TopicV2 {
        TopicV2 {
            id,
            sub_topics: Vec::new(),
        }
    }

    pub fn add_sub_topic(&mut self, sub_topic: TopicV2) {
        self.sub_topics.push(sub_topic);
    }

    pub fn create_topicsGPT(path: &str, root: &mut TopicV2) -> u64 {
        let mut last_created_topic_id = root.id;
        let topic_names: Vec<&str> = path.split("/").collect();

        let mut current_topic = root;
        for topic_name in topic_names {
            if topic_name.is_empty() {
                continue;
            }
            let topic_id = {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                topic_name.hash(&mut hasher);
                hasher.finish()
            };

            let existing_topic_idx = current_topic.sub_topics.iter().position(|t| t.id == topic_id);
            if let Some(idx) = existing_topic_idx {
                current_topic = &mut current_topic.sub_topics[idx];
            } else {
                let new_topic = TopicV2::new(topic_id);
                current_topic.sub_topics.push(new_topic);
                let new_topic_idx = current_topic.sub_topics.len() - 1;
                current_topic = &mut current_topic.sub_topics[new_topic_idx];
            }
            last_created_topic_id = topic_id;
        }
        last_created_topic_id
    }

/*
    pub fn create_topics(path: &str, root: &mut TopicV2) -> u64 {

        let mut last_created_topic_id = root.id;
        let topic_names: Vec<&str> = path.split("/").collect();

        let mut current_topic = root;
        for topic_name in topic_names {
            if topic_name.is_empty() {
                continue;
            }
            let topic_id = {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                topic_name.hash(&mut hasher);
                hasher.finish()
            };

            let test = match current_topic.sub_topics.get_mut(&topic_id) {
                None => {
                    let mut new_topic = TopicV2::new(topic_id);

                    current_topic.sub_topics.insert(topic_id, new_topic.clone());
                    current_topic.sub_topics.get_mut(&topic_id).unwrap()
                },
                Some(existing_topic) => existing_topic,
            };

            /*if let Some(existing_topic) = current_topic.sub_topics.get_mut(&topic_id) {
                current_topic = existing_topic;
            } else {
                let mut new_topic = TopicV2::new(topic_id);

                current_topic.sub_topics.insert(topic_id, new_topic.clone());
                current_topic = current_topic.sub_topics.get_mut(&topic_id).unwrap();
            }*/
            last_created_topic_id = topic_id;
        }
        last_created_topic_id
    }
 */
}
