use crate::codec::message_id::MessageId;

#[derive(Debug)]
pub struct MessageIdStore {
    claimed: Vec<MessageId>,
    next: Option<MessageId>,
}

impl MessageIdStore {
    pub fn new(initial_value: MessageId) -> Self {
        Self {
            claimed: Default::default(),
            next: Some(initial_value),
        }
    }

    pub fn at_capacity(&self) -> bool {
        self.next.is_none()
    }

    pub fn claim(&mut self) -> Option<MessageId> {
        let claimed = match self.next {
            Some(next) => next,
            None => return None,
        };

        let next = claimed.next();
        if self.is_claimed(&next) {
            self.next = None;
        } else {
            self.next = Some(next);
        }

        self.claimed.push(claimed);

        Some(claimed)
    }

    pub fn release(&mut self, message_id: MessageId) {
        let position = match self.claimed.iter().position(|m| *m == message_id) {
            Some(position) => position,
            None => return,
        };

        self.claimed.swap_remove(position);
        if self.next.is_none() {
            self.next = Some(message_id)
        }
    }

    pub fn is_claimed(&self, message_id: &MessageId) -> bool {
        self.claimed.contains(message_id)
    }
}
