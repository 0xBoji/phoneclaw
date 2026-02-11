use crate::types::Message;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum Event {
    InboundMessage(Message),
    OutboundMessage(Message),
    SystemLog { level: String, message: String },
}

pub struct MessageBus {
    tx: broadcast::Sender<Event>,
}

impl MessageBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: Event) -> Result<usize, broadcast::error::SendError<Event>> {
        self.tx.send(event)
    }
}
