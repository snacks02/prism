use {
    composition::{
        Composition,
        main,
    },
    iced::{
        Color,
        Element,
        Result,
        Settings,
        Subscription,
        Task,
        Theme,
        theme::palette::Seed,
    },
};

mod audio_player;
mod composition;
mod icon;
mod list;
mod queue;
mod style;
mod track;
mod view_helper;

const DEFAULT_TEXT_SIZE: f32 = 14.0;

#[derive(Clone, Debug)]
enum Message {
    Main(main::Message),
}

fn main() -> Result {
    iced::application(Prism::new, Prism::update, Prism::view)
        .settings(Settings {
            default_text_size: DEFAULT_TEXT_SIZE.into(),
            ..Default::default()
        })
        .subscription(Prism::subscription)
        .theme(Prism::theme)
        .title("Prism")
        .run()
}

impl Composition for Prism {
    fn new() -> Self {
        Self {
            color_primary: style::COLOR_PRIMARY,
            main: main::Main::new(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        self.main.subscription().map(Message::Main)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Main(message) => match self.main.update(message) {
                main::Event::None => Task::none(),
                main::Event::Perform(task) => task.map(Message::Main),
                main::Event::PrimaryColorChange(color) => {
                    self.color_primary = color;
                    Task::none()
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        self.main.view().map(Message::Main)
    }

    type Event = Task<Message>;

    type Message = Message;
}

impl Prism {
    fn theme(&self) -> Theme {
        Theme::custom(
            "Prism".to_string(),
            Seed {
                background: style::COLOR_BACKGROUND,
                primary: self.color_primary,
                text: style::COLOR_GRAY_4,
                ..Seed::DARK
            },
        )
    }
}

struct Prism {
    color_primary: Color,
    main: main::Main,
}
