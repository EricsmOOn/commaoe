use anyhow::Result;
use iced::alignment::Horizontal;
use iced::executor;
use iced::widget::{button, column, container, row, text, text_input};
use iced::{window, Alignment, Application, Command, Element, Font, Settings, Theme};
use once_cell::sync::Lazy;
use parse::Proxies;
use reqwest::Client;
use serde_json::Value;

static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

// Fonts
pub const FONT: Font = Font::External {
    name: "font",
    bytes: include_bytes!("../font/SmileySans-Oblique.ttf"),
};

pub fn main() -> iced::Result {
    Commaoe::run(Settings {
        antialiasing: true,
        window: window::Settings {
            size: (600, 800),
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
struct Commaoe {
    base: Base,
    content: String,
    proxies: Proxies,
}

#[derive(Debug, Clone)]
struct Base {
    ip: String,
    port: String,
    client: Client,
}

#[derive(Debug, Clone)]
pub enum Message {
    IPChanged(String),
    PortChanged(String),
    FetchData,
    AllProxies(Option<Proxies>),
}

impl Default for Commaoe {
    fn default() -> Self {
        let client = Client::new();
        let ip = String::from("http://127.0.0.1");
        let port = String::from("9090");
        let base = Base { ip, port, client };
        let proxies = Default::default();
        let content = Default::default();
        Self {
            base,
            proxies,
            content,
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
        String::from("Clash Info")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IPChanged(value) => {
                self.base.ip = value;
                Command::none()
            }
            Message::PortChanged(value) => {
                self.base.port = value;
                Command::none()
            }
            Message::FetchData => {
                let base = self.base.clone();
                Command::perform(async move { all_proxy(base).await }, Message::AllProxies)
            }
            Message::AllProxies(data) => {
                if let Some(proxies) = data {
                    self.proxies = proxies
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let ip = container(row![
            text("URL")
                .font(FONT)
                .width(60)
                .horizontal_alignment(Horizontal::Center),
            text_input("IP", &self.base.ip)
                .font(FONT)
                .id(INPUT_ID.clone())
                .on_input(Message::IPChanged)
                .width(200)
                .size(20),
        ])
        .align_x(Horizontal::Left)
        .width(250);
        let port = container(row![
            text("Port")
                .font(FONT)
                .width(60)
                .horizontal_alignment(Horizontal::Center),
            text_input("PORT", &self.base.port)
                .font(FONT)
                .id(INPUT_ID.clone())
                .on_input(Message::PortChanged)
                .width(60)
                .size(20),
        ])
        .align_x(Horizontal::Left)
        .width(140);
        let lookup = container(button("LOOKUP").on_press(Message::FetchData).width(80));
        let setting = row![ip, port, lookup,].padding(40);
        let content = column![text(&self.content).size(50), self.proxies.view(),]
            .align_items(Alignment::Center);
        column![setting, content].into()
    }
}

async fn fetch_data(base: Base, path: String) -> Result<Value> {
    let url = format!("{}:{}/{}", &base.ip, &base.port, &path);
    let resp = base.client.get(url).send().await?;
    let result: Value = resp.json().await?;
    Ok(result)
}

async fn all_proxy(base: Base) -> Option<Proxies> {
    let path = String::from("proxies");
    let json = fetch_data(base, path).await;
    match json {
        Ok(json) => Proxies::from_value(json),
        Err(e) => {
            print!("error {:?}", e);
            None
        }
    }
}

mod parse {
    use crate::Message;
    use crate::FONT;
    use iced::alignment::Horizontal;
    use iced::widget::Row;
    use iced::widget::{container, scrollable, text, Column};
    use iced::{Element, Length};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::collections::HashMap;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct History {
        time: String,
        delay: i32,
        #[serde(rename = "meanDelay")]
        mean_delay: i32,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct Proxy {
        history: Vec<History>,
        name: String,
        #[serde(rename = "type")]
        proxy_type: String,
        udp: bool,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Proxies {
        proxies: HashMap<String, Proxy>,
    }

    impl Default for Proxies {
        fn default() -> Self {
            Self {
                proxies: HashMap::new(),
            }
        }
    }

    impl Proxies {
        pub fn from_value(json: Value) -> Option<Self> {
            match serde_json::from_value::<Self>(json) {
                Ok(parsed_data) => Some(parsed_data),
                Err(e) => {
                    print!("error {:?}", e);
                    None
                }
            }
        }

        pub fn view(&self) -> Element<Message> {
            let mut sorted_proxies = self.proxies.iter().collect::<Vec<_>>();
            sorted_proxies.sort_by(|(_, p2), (_, p1)| {
                (p1.proxy_type.clone(), p1.name.clone())
                    .cmp(&(p2.proxy_type.clone(), p2.name.clone()))
            });

            let proxies =
                Column::with_children(sorted_proxies.iter().map(|(_, p)| Proxy::view(p)).collect());

            scrollable(
                container(proxies)
                    .width(Length::Fill)
                    .padding(10)
                    .center_y(),
            )
            .into()
        }
    }

    impl Proxy {
        pub fn view(&self) -> Element<Message> {
            Row::new()
                // .push(button("choose").padding(10))
                .push(
                    text(self.name.to_owned())
                        .font(FONT)
                        .horizontal_alignment(Horizontal::Center)
                        .width(Length::FillPortion(3))
                        .size(20),
                )
                .push(
                    text(self.proxy_type.to_owned())
                        .font(FONT)
                        .horizontal_alignment(Horizontal::Center)
                        .width(Length::FillPortion(3))
                        .size(20),
                )
                .into()
        }
    }
}
