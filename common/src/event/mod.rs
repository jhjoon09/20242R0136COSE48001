use crate::message::Message;

pub trait Event<T: Message>: Send + 'static {
    fn from_message(message: T) -> Self;
}
