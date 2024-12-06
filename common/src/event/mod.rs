use crate::message::Message;
pub mod client;

pub trait Event<T: Message>: Send + 'static {
    fn from_message(message: T) -> Self;
}
