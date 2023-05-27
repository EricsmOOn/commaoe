use iced::executor;
use iced::widget::{button, column, text, text_input};
use iced::{Alignment, Application, Command, Element, Settings, Theme};
use once_cell::sync::Lazy;
use reqwest::Client;

static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub fn main() -> iced::Result {
    Commaoe::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Debug, Clone)]
struct Commaoe {
    ip: String,
    port: String,
    client: Client,
    content: String,
}

#[derive(Debug, Clone)]
enum Message {
    IPChanged(String),
    PortChanged(String),
    FetchData,
    DataFetched(String),
}

impl Default for Commaoe {
    fn default() -> Self {
        let client = Client::new();
        Self {
            ip: String::from("http://127.0.0.1"),
            port: String::from("9090"),
            content: String::default(),
            client,
        }
    }
}

impl Application for Commaoe {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Commaoe, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("commaOeRic")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IPChanged(value) => {
                self.ip = value;
                Command::none()
            }
            Message::PortChanged(value) => {
                self.port = value;
                Command::none()
            }
            Message::FetchData => {
                let client = self.client.clone();
                let url = format!("{}:{}", self.ip, self.port);
                Command::perform(
                    async move {
                        let response = client.get(url).send().await;
                        match response {
                            Ok(resp) => match resp.text().await {
                                Ok(json) => json,
                                Err(e) => e.to_string(),
                            },
                            Err(e) => e.to_string(),
                        }
                    },
                    Message::DataFetched,
                )
            }
            Message::DataFetched(data) => {
                self.content = data;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            text_input("IP", &self.ip)
                .id(INPUT_ID.clone())
                .on_input(Message::IPChanged)
                .size(50),
            text_input("PORT", &self.port)
                .id(INPUT_ID.clone())
                .on_input(Message::PortChanged)
                .size(50),
            text(&self.content).size(50),
            button("LOOKUP").on_press(Message::FetchData),
        ]
        .padding(40)
        .align_items(Alignment::Center)
        .into()
    }
}
