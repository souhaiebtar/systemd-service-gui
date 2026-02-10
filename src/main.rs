mod systemd;

use iced::{
    theme,
    widget::{text_input, Button, Column, Container, Row, Scrollable, Text},
    Alignment, Application, Command, Element, Length, Settings, Theme,
};
use systemd::{list_services, ServiceInfo, start_service, stop_service, restart_service};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusFilter {
    Running,
    Exited,
    Dead,
    Active,
    Inactive,
}

#[derive(Debug, Clone)]
enum Message {
    RefreshServices,
    FilterChanged(String),
    ToggleStatusFilter(StatusFilter),
    StartService(String),
    StopService(String),
    RestartService(String),
    ServicesLoaded(Result<Vec<ServiceInfo>, String>),
}

struct SystemdServiceGui {
    services: Vec<ServiceInfo>,
    name_filter: String,
    status_filter: Option<StatusFilter>,
    loading: bool,
    error: Option<String>,
}

impl Application for SystemdServiceGui {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let app = SystemdServiceGui {
            services: Vec::new(),
            name_filter: String::new(),
            status_filter: None,
            loading: false,
            error: None,
        };

        let command = app.load_services();
        (app, command)
    }

    fn title(&self) -> String {
        String::from("Systemd Service GUI")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::RefreshServices => self.refresh(),
            Message::FilterChanged(value) => {
                self.name_filter = value;
                Command::none()
            }
            Message::ToggleStatusFilter(filter) => {
                self.status_filter = match self.status_filter {
                    Some(selected) if selected == filter => None,
                    _ => Some(filter),
                };
                Command::none()
            }
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

    fn view(&self) -> Element<'_, Message> {
        let title = Text::new("Systemd Service GUI")
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

        let name_filter_input = text_input("Filter services by name...", &self.name_filter)
            .on_input(Message::FilterChanged)
            .padding(10)
            .size(16)
            .width(Length::Fill);

        let status_filter_row = Row::new()
            .push(Text::new("Status:"))
            .push(self.status_filter_button("running", StatusFilter::Running))
            .push(self.status_filter_button("exited", StatusFilter::Exited))
            .push(self.status_filter_button("dead", StatusFilter::Dead))
            .push(self.status_filter_button("active", StatusFilter::Active))
            .push(self.status_filter_button("inactive", StatusFilter::Inactive))
            .spacing(10)
            .align_items(Alignment::Center)
            .width(Length::Fill);

        let mut content = Column::new().spacing(10);
        let filtered_services = self.filtered_services();

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
        } else if filtered_services.is_empty() {
            content = content.push(Text::new("No services match the current filters.").size(16));
        } else {
            for service in filtered_services {
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
                .push(name_filter_input)
                .push(status_filter_row)
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

impl SystemdServiceGui {
    fn status_filter_button<'a>(&self, label: &'a str, filter: StatusFilter) -> Button<'a, Message> {
        let is_selected = self.status_filter == Some(filter);
        Button::new(Text::new(label))
            .on_press(Message::ToggleStatusFilter(filter))
            .style(if is_selected {
                theme::Button::Primary
            } else {
                theme::Button::Secondary
            })
    }

    fn filtered_services(&self) -> Vec<&ServiceInfo> {
        let needle = self.name_filter.trim().to_ascii_lowercase();
        self.services
            .iter()
            .filter(|service| {
                let name_ok = needle.is_empty()
                    || service.name.to_ascii_lowercase().contains(&needle);
                let status_ok = self
                    .status_filter
                    .map(|status| matches_status_filter(service, status))
                    .unwrap_or(true);

                name_ok && status_ok
            })
            .collect()
    }

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

fn matches_status_filter(service: &ServiceInfo, filter: StatusFilter) -> bool {
    match filter {
        StatusFilter::Running => service.sub_state.eq_ignore_ascii_case("running"),
        StatusFilter::Exited => service.sub_state.eq_ignore_ascii_case("exited"),
        StatusFilter::Dead => service.sub_state.eq_ignore_ascii_case("dead"),
        StatusFilter::Active => service.active_state.eq_ignore_ascii_case("active"),
        StatusFilter::Inactive => service.active_state.eq_ignore_ascii_case("inactive"),
    }
}

pub fn main() -> iced::Result {
    SystemdServiceGui::run(Settings::default())
}
