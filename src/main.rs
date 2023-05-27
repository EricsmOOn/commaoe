use anyhow::Result;
use iced::executor;
use iced::widget::{button, column, text, text_input};
use iced::{Alignment, Application, Command, Element, Settings, Theme};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
struct Proxies {
    proxies: HashMap<String, Proxy>,
}

#[derive(Debug, Clone)]
enum Message {
    IPChanged(String),
    PortChanged(String),
    FetchData,
    AllProxies(Option<Proxies>),
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
                let ip = self.ip.clone();
                let port = self.port.clone();
                Command::perform(
                    async move { all_proxy(client, ip, port).await },
                    Message::AllProxies,
                )
            }
            Message::AllProxies(data) => {
                if let Some(strs) = data {
                    self.content = format!("{:?}", strs)
                }
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

async fn fetch_data(client: Client, ip: String, port: String, path: String) -> Result<Value> {
    let url = format!("{}:{}/{}", &ip, &port, &path);
    let resp = client.get(url).send().await?;
    let result: Value = resp.json().await?;
    Ok(result)
}

async fn all_proxy(client: Client, ip: String, port: String) -> Option<Proxies> {
    let path = String::from("proxies");
    let json = fetch_data(client.clone(), ip.to_owned(), port.to_owned(), path).await;
    if let Ok(json) = json {
        if let Ok(parsed_data) = serde_json::from_value::<Proxies>(json) {
            Some(parsed_data)
        } else {
            None
        }
    } else {
        None
    }
  }
