use iced::{
    tab, Align, Column, Container, Element, Length, Row, Rule, Sandbox,
    Settings, Tab, Text, VerticalAlignment,
};

pub fn main() -> iced::Result {
    Tabs::run(Settings::default())
}

struct Tabs {
    page_selection: Page,
    v_menu_selection: VerticalMenu,
}

impl Default for Tabs {
    fn default() -> Self {
        Tabs {
            page_selection: Page::Home,
            v_menu_selection: VerticalMenu::IAm,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    PageSelected(Page),
    VMenuSelected(VerticalMenu),
}

impl Sandbox for Tabs {
    type Message = Message;

    fn new() -> Self {
        Tabs::default()
    }

    fn title(&self) -> String {
        String::from("Tabs - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::PageSelected(page) => {
                self.page_selection = page;
            }
            Message::VMenuSelected(selection) => {
                self.v_menu_selection = selection;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let page_tabs = Page::all().iter().cloned().fold(
            Row::new().padding(0).spacing(10),
            |choices, page| {
                choices.push(Tab::new(
                    page,
                    Some(self.page_selection),
                    Message::PageSelected,
                    Text::new(page).size(24),
                ))
            },
        );

        let content = Column::new()
            .spacing(20)
            .padding(20)
            .push(Column::new().push(page_tabs).push(Rule::horizontal(0)))
            .push(match self.page_selection {
                Page::Home => Container::new(Text::new("Home sweet home!"))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y(),
                Page::Vertical => {
                    let v_menu_tabs = VerticalMenu::all().iter().cloned().fold(
                        Column::new()
                            .padding(0)
                            .spacing(0)
                            .width(Length::Units(110))
                            .push(Rule::horizontal(0)),
                        |choices, page| {
                            choices
                                .push(
                                    Tab::new(
                                        page,
                                        Some(self.v_menu_selection),
                                        Message::VMenuSelected,
                                        Text::new(page)
                                            .size(20)
                                            .vertical_alignment(
                                                VerticalAlignment::Center,
                                            )
                                            .width(Length::Fill),
                                    )
                                    .width(Length::Fill)
                                    .padding(5)
                                    .style(tab::StyleDefaultVertical),
                                )
                                .push(Rule::horizontal(0))
                        },
                    );

                    Container::new(
                        Row::new()
                            .align_items(Align::Center)
                            .push(v_menu_tabs)
                            .push(
                                Container::new(match self.v_menu_selection {
                                    VerticalMenu::IAm => Text::new("I am"),
                                    VerticalMenu::A => Text::new("a"),
                                    VerticalMenu::Vertical => {
                                        Text::new("vertical")
                                    }
                                    VerticalMenu::TabMenu => {
                                        Text::new("tab menu")
                                    }
                                })
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .center_x()
                                .center_y(),
                            ),
                    )
                }
                Page::About => {
                    Container::new(Text::new("Nothing to see here."))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x()
                        .center_y()
                }
            });

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Vertical,
    About,
}

impl Page {
    fn all() -> [Page; 3] {
        [Page::Home, Page::Vertical, Page::About]
    }
}

impl From<Page> for String {
    fn from(page: Page) -> String {
        String::from(match page {
            Page::Home => "Home",
            Page::Vertical => "Vertical",
            Page::About => "About",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalMenu {
    IAm,
    A,
    Vertical,
    TabMenu,
}

impl VerticalMenu {
    fn all() -> [VerticalMenu; 4] {
        [
            VerticalMenu::IAm,
            VerticalMenu::A,
            VerticalMenu::Vertical,
            VerticalMenu::TabMenu,
        ]
    }
}

impl From<VerticalMenu> for String {
    fn from(page: VerticalMenu) -> String {
        String::from(match page {
            VerticalMenu::IAm => "I am",
            VerticalMenu::A => "a",
            VerticalMenu::Vertical => "vertical",
            VerticalMenu::TabMenu => "tab menu",
        })
    }
}
