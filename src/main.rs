mod systemd;

use iced::{
    widget::{button, scrollable, text, Button, Column, Container, Row, Scrollable, Text},
    Alignment, Application, Command, Element, Length, Settings, Theme,
};
use systemd::{list_services, ServiceInfo, start_service, stop_service, restart_service};

#[derive(Debug, Clone)]
enum Message {
    RefreshServices,
    StartService(String),
    StopService(String),
    RestartService(String),
    ServicesLoaded(Result<Vec<ServiceInfo>, String>),
}

struct IcErs {
    services: Vec<ServiceInfo>,
    loading: bool,
    error: Option<String>,
}

impl Application for IcErs {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let app = IcErs {
            services: Vec::new(),
            loading: false,
            error: None,
        };

        let command = app.load_services();
        (app, command)
    }

    fn title(&self) -> String {
        String::from("IcErs - Systemd Service Manager")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::RefreshServices => self.refresh(),
            Message::StartService(name) => self.start(name),
            Message::StopService(name) => self.stop(name),
            Message::RestartService(name) => self.restart(name),
            Message::ServicesLoaded(result) => {
                self.loading = false;
                match result {
                    Ok(services) => {
                        self.services = services;
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let title = Text::new("IcErs - Systemd Service Manager")
            .size(40)
            .width(Length::Fill);

        let refresh_button = Button::new(
            Text::new("Refresh"),
        )
        .on_press(Message::RefreshServices);

        let header = Row::new()
            .push(title)
            .push(refresh_button)
            .align_items(Alignment::Center)
            .spacing(10)
            .width(Length::Fill);

        let mut content = Column::new().spacing(10);

        if let Some(error) = &self.error {
            content = content.push(
                Text::new(format!("Error: {}", error))
                    .size(16),
            );
        }

        if self.loading {
            content = content.push(Text::new("Loading services...").size(16));
        } else if self.services.is_empty() {
            content = content.push(Text::new("No services found or unable to load services.").size(16));
        } else {
            for service in &self.services {
                let status_text = if service.is_active() {
                    "active"
                } else {
                    "inactive"
                };

                let service_row = Row::new()
                    .push(
                        Text::new(format!("{}", service.name))
                            .width(Length::Fixed(250.0))
                    )
                    .push(
                        Text::new(format!("{}", service.description))
                            .width(Length::Fixed(300.0))
                    )
                    .push(
                        Text::new(format!("{}", service.active_state))
                            .width(Length::Fixed(100.0))
                    )
                    .push(
                        Text::new(format!("{}", service.sub_state))
                            .width(Length::Fixed(100.0))
                    )
                    .push(
                        Button::new(
                            Text::new("Start"),
                        )
                        .on_press(Message::StartService(service.name.clone()))
                    )
                    .push(
                        Button::new(
                            Text::new("Stop"),
                        )
                        .on_press(Message::StopService(service.name.clone()))
                    )
                    .push(
                        Button::new(
                            Text::new("Restart"),
                        )
                        .on_press(Message::RestartService(service.name.clone()))
                    )
                    .spacing(10)
                    .align_items(Alignment::Center);

                content = content.push(service_row);
            }
        }

        let scroll_content = Scrollable::new(content)
            .width(Length::Fill)
            .height(Length::Fill);

        Container::new(
            Column::new()
                .push(header)
                .push(scroll_content)
                .spacing(20)
                .padding(20)
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

impl IcErs {
    fn load_services(&self) -> Command<Message> {
        Command::perform(
            async {
                list_services()
            },
            Message::ServicesLoaded,
        )
    }

    fn refresh(&self) -> Command<Message> {
        self.load_services()
    }

    fn start(&self, name: String) -> Command<Message> {
        Command::perform(
            async move {
                start_service(&name).map(|_| ())
            },
            |result| {
                match result {
                    Ok(_) => Message::ServicesLoaded(list_services()),
                    Err(e) => Message::ServicesLoaded(Err(e)),
                }
            },
        )
    }

    fn stop(&self, name: String) -> Command<Message> {
        Command::perform(
            async move {
                stop_service(&name).map(|_| ())
            },
            |result| {
                match result {
                    Ok(_) => Message::ServicesLoaded(list_services()),
                    Err(e) => Message::ServicesLoaded(Err(e)),
                }
            },
        )
    }

    fn restart(&self, name: String) -> Command<Message> {
        Command::perform(
            async move {
                restart_service(&name).map(|_| ())
            },
            |result| {
                match result {
                    Ok(_) => Message::ServicesLoaded(list_services()),
                    Err(e) => Message::ServicesLoaded(Err(e)),
                }
            },
        )
    }
}

pub fn main() -> iced::Result {
    IcErs::run(Settings::default())
}
