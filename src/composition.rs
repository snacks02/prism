use iced::{
    Element,
    Subscription,
};

pub trait Composition {
    fn new() -> Self;

    fn subscription(&self) -> Subscription<Self::Message>;

    #[must_use]
    fn update(&mut self, message: Self::Message) -> Self::Event;

    #[must_use]
    fn view(&self) -> Element<'_, Self::Message>;

    type Event;

    type Message: Clone;
}
