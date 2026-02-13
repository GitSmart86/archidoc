// Event dispatch and subscription
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

pub trait EventBus {
    fn subscribe(&mut self, handler: Box<dyn Fn(String)>);
    fn publish(&self, event: String);
}
