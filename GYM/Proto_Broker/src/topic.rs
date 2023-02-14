pub struct Topic {
    topics: Vec<Topic>,
    id: u64,
}

impl Topic {

    pub fn new(id: u64) -> Self {
        Topic {
            topics: Default::default(),
            id,
        }
    }

    pub fn get_sub_topic_by_id(&self, id: u64) -> Option<&Topic> {
        // using Option as it's much cleaner
        if self.id == id {
            Some(self)
        } else {
            // iterate over sub topics with mutable ref
            for topic in self.topics.iter() {
                // let to dynamically declare boolean
                if let Some(new_topic) = topic.get_sub_topic_by_id(id) {
                    return Some(new_topic);
                }
            }
            None
        }
    }

    pub fn add_sub_topic(&mut self, topic: Topic) {
        self.topics.push(topic);
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
}