pub struct Topic {
    topics: Vec<Topic>,
    id: u64,
}

impl Topic {
    pub fn create_hash() {}


    pub fn new(id: u64) -> Self {
        Topic {
            topics: Default::default(),
            id,
        }
    }

    pub fn get_sub_topic_by_id(&mut self, id: u64) -> Option<&mut Topic> {
        if self.id == id {
            return Some(self);
        } else {
            for topic in &mut self.topics {
                let new_topic = topic.get_sub_topic_by_id(id);
                if new_topic.is_some() {
                    return new_topic;
                }
            }
        }
        return None;
    }
    pub fn add_sub_topic(&mut self, topic: Topic) {
        self.topics.push(topic);
    }

    pub fn get_sub_topics(&self) -> &Vec<Topic> {
        return &self.topics;
    }

    pub fn get_id(&self) -> u64 {
        return self.id;
    }
}