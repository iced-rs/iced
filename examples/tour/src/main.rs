use iced::font;
use iced::widget::{Button, Column, Container, Slider};
use iced::widget::{
    button, center_x, center_y, checkbox, column, container, image, radio, rich_text, row,
    scrollable, slider, space, span, text, text_input, toggler, transition,
};
use iced::{Animation, Center, Color, Element, Fill, Font, Pixels, Theme, color};

use std::time::Duration;

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init().expect("Initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    iced::application(Tour::default, Tour::update, Tour::view)
        .title(Tour::title)
        .subscription(Tour::subscription)
        .centered()
        .run()
}

pub struct Tour {
    screen: Screen,
    slider: u8,
    layout: Layout,
    spacing: u32,
    text_size: u32,
    text_color: Color,
    language: Option<Language>,
    toggler: bool,
    image_width: u32,
    image_filter_method: image::FilterMethod,
    input_value: String,
    input_is_secure: bool,
    input_is_showing_icon: bool,
    debug: bool,
    udhr_language: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackPressed,
    NextPressed,
    SliderChanged(u8),
    LayoutChanged(Layout),
    SpacingChanged(u32),
    TextSizeChanged(u32),
    TextColorChanged(Color),
    LanguageSelected(Language),
    ImageWidthChanged(u32),
    ImageUseNearestToggled(bool),
    InputChanged(String),
    ToggleSecureInput(bool),
    ToggleTextInputIcon(bool),
    DebugToggled(bool),
    TogglerChanged(bool),
    NextLanguage,
    OpenTrunk,
    OpenDeclaration,
}

impl Tour {
    fn title(&self) -> String {
        let screen = match self.screen {
            Screen::Welcome => "Welcome",
            Screen::Radio => "Radio button",
            Screen::Toggler => "Toggler",
            Screen::Slider => "Slider",
            Screen::Text => "Text",
            Screen::RichText => "Rich text",
            Screen::Image => "Image",
            Screen::RowsAndColumns => "Rows and columns",
            Screen::Scrollable => "Scrollable",
            Screen::TextInput => "Text input",
            Screen::Debugger => "Debugger",
            Screen::End => "End",
        };

        format!("{screen} - Iced")
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::BackPressed => {
                if let Some(screen) = self.screen.previous() {
                    self.screen = screen;
                }
            }
            Message::NextPressed => {
                if let Some(screen) = self.screen.next() {
                    self.screen = screen;
                }
            }
            Message::SliderChanged(value) => {
                self.slider = value;
            }
            Message::LayoutChanged(layout) => {
                self.layout = layout;
            }
            Message::SpacingChanged(spacing) => {
                self.spacing = spacing;
            }
            Message::TextSizeChanged(text_size) => {
                self.text_size = text_size;
            }
            Message::TextColorChanged(text_color) => {
                self.text_color = text_color;
            }
            Message::LanguageSelected(language) => {
                self.language = Some(language);
            }
            Message::ImageWidthChanged(image_width) => {
                self.image_width = image_width;
            }
            Message::ImageUseNearestToggled(use_nearest) => {
                self.image_filter_method = if use_nearest {
                    image::FilterMethod::Nearest
                } else {
                    image::FilterMethod::Linear
                };
            }
            Message::InputChanged(input_value) => {
                self.input_value = input_value;
            }
            Message::ToggleSecureInput(is_secure) => {
                self.input_is_secure = is_secure;
            }
            Message::ToggleTextInputIcon(show_icon) => {
                self.input_is_showing_icon = show_icon;
            }
            Message::DebugToggled(debug) => {
                self.debug = debug;
            }
            Message::TogglerChanged(toggler) => {
                self.toggler = toggler;
            }
            Message::NextLanguage => {
                self.udhr_language += 1;
            }
            Message::OpenTrunk => {
                #[cfg(not(target_arch = "wasm32"))]
                let _ = open::that_in_background("https://trunkrs.dev");
            }
            Message::OpenDeclaration => {
                #[cfg(not(target_arch = "wasm32"))]
                let _ = open::that_in_background(
                    "https://www.un.org/en/about-us/universal-declaration-of-human-rights",
                );
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        if self.screen == Screen::RichText {
            iced::time::every(Duration::from_secs(6)).map(|_| Message::NextLanguage)
        } else {
            iced::Subscription::none()
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            self.screen.previous().is_some().then(|| {
                padded_button("Back")
                    .on_press(Message::BackPressed)
                    .style(button::secondary)
            }),
            space::horizontal(),
            self.can_continue()
                .then(|| { padded_button("Next").on_press(Message::NextPressed) })
        ];

        let screen = match self.screen {
            Screen::Welcome => self.welcome(),
            Screen::Radio => self.radio(),
            Screen::Toggler => self.toggler(),
            Screen::Slider => self.slider(),
            Screen::Text => self.text(),
            Screen::RichText => self.rich_text(),
            Screen::Image => self.image(),
            Screen::RowsAndColumns => self.rows_and_columns(),
            Screen::Scrollable => self.scrollable(),
            Screen::TextInput => self.text_input(),
            Screen::Debugger => self.debugger(),
            Screen::End => self.end(),
        };

        let content: Element<_> = column![screen, controls].max_width(540).spacing(20).into();

        let scrollable = scrollable(center_x(if self.debug {
            content.explain(Color::BLACK)
        } else {
            content
        }))
        .spacing(10)
        .auto_scroll(true);

        center_y(scrollable).padding(10).into()
    }

    fn can_continue(&self) -> bool {
        match self.screen {
            Screen::Welcome => true,
            Screen::Radio => self.language == Some(Language::Rust),
            Screen::Toggler => self.toggler,
            Screen::Slider => true,
            Screen::Text => true,
            Screen::RichText => true,
            Screen::Image => true,
            Screen::RowsAndColumns => true,
            Screen::Scrollable => true,
            Screen::TextInput => !self.input_value.is_empty(),
            Screen::Debugger => true,
            Screen::End => false,
        }
    }

    fn welcome(&self) -> Column<'_, Message> {
        Self::container("Welcome!")
            .push(
                "This is a simple tour meant to showcase a bunch of \
                widgets that come bundled in Iced.",
            )
            .push(
                "Iced is a cross-platform GUI library for Rust focused on \
                 simplicity and type-safety. It is heavily inspired by Elm.",
            )
            .push(
                "It was originally born as part of Coffee, an opinionated \
                 2D game engine for Rust.",
            )
            .push(
                "On native platforms, Iced provides by default a renderer \
                 built on top of wgpu, a graphics library supporting Vulkan, \
                 Metal, DX11, and DX12.",
            )
            .push(
                rich_text![
                    "Additionally, this tour can also run on WebAssembly ",
                    "by leveraging ",
                    span("trunk")
                        .color(color!(0x7777FF))
                        .underline(true)
                        .font(Font::MONOSPACE)
                        .link(Message::OpenTrunk),
                    "."
                ]
                .on_link_click(std::convert::identity),
            )
            .push(
                "You will need to interact with the UI in order to reach \
                 the end!",
            )
    }

    fn slider(&self) -> Column<'_, Message> {
        Self::container("Slider")
            .push(
                "A slider allows you to smoothly select a value from a range \
                 of values.",
            )
            .push(
                "The following slider lets you choose an integer from \
                 0 to 100:",
            )
            .push(slider(0..=100, self.slider, Message::SliderChanged))
            .push(text(self.slider.to_string()).width(Fill).align_x(Center))
    }

    fn rows_and_columns(&self) -> Column<'_, Message> {
        let row_radio = radio(
            "Row",
            Layout::Row,
            Some(self.layout),
            Message::LayoutChanged,
        );

        let column_radio = radio(
            "Column",
            Layout::Column,
            Some(self.layout),
            Message::LayoutChanged,
        );

        let layout_section: Element<_> = match self.layout {
            Layout::Row => row![row_radio, column_radio].spacing(self.spacing).into(),
            Layout::Column => column![row_radio, column_radio]
                .spacing(self.spacing)
                .into(),
        };

        let spacing_section = column![
            slider(0..=80, self.spacing, Message::SpacingChanged),
            text!("{} px", self.spacing).width(Fill).align_x(Center),
        ]
        .spacing(10);

        Self::container("Rows and columns")
            .spacing(self.spacing)
            .push(
                "Iced uses a layout model based on flexbox to position UI \
                 elements.",
            )
            .push(
                "Rows and columns can be used to distribute content \
                 horizontally or vertically, respectively.",
            )
            .push(layout_section)
            .push("You can also easily change the spacing between elements:")
            .push(spacing_section)
    }

    fn text(&self) -> Column<'_, Message> {
        let size = self.text_size;
        let color = self.text_color;

        let size_section = column![
            "You can change its size:",
            text!("This text is {size} pixels").size(size),
            slider(3..=70, size, Message::TextSizeChanged),
        ]
        .padding(20)
        .spacing(20);

        let color_sliders = row![
            color_slider(color.r, move |r| Color { r, ..color }),
            color_slider(color.g, move |g| Color { g, ..color }),
            color_slider(color.b, move |b| Color { b, ..color }),
        ]
        .spacing(10);

        let color_section = column![
            "And its color:",
            text!("{color:?}").color(color),
            color_sliders,
        ]
        .padding(20)
        .spacing(20);

        Self::container("Text")
            .push(
                "Text is probably the most essential widget for your UI. \
                 It will try to adapt to the dimensions of its container.",
            )
            .push(size_section)
            .push(color_section)
    }

    fn radio(&self) -> Column<'_, Message> {
        let question = column![
            text("Iced is written in...").size(24),
            column(
                Language::all()
                    .iter()
                    .copied()
                    .map(|language| {
                        radio(language, language, self.language, Message::LanguageSelected)
                    })
                    .map(Element::from)
            )
            .spacing(10)
        ]
        .padding(20)
        .spacing(10);

        Self::container("Radio button")
            .push(
                "A radio button is normally used to represent a choice... \
                 Surprise test!",
            )
            .push(question)
            .push(
                "Iced works very well with iterators! The list above is \
                 basically created by folding a column over the different \
                 choices, creating a radio button for each one of them!",
            )
    }

    fn toggler(&self) -> Column<'_, Message> {
        Self::container("Toggler")
            .push("A toggler is mostly used to enable or disable something.")
            .push(
                Container::new(
                    toggler(self.toggler)
                        .label("Toggle me to continue...")
                        .on_toggle(Message::TogglerChanged),
                )
                .padding([0, 40]),
            )
    }

    fn image(&self) -> Column<'_, Message> {
        let width = self.image_width;
        let filter_method = self.image_filter_method;

        Self::container("Image")
            .push("An image that tries to keep its aspect ratio.")
            .push(ferris(width, filter_method))
            .push(slider(100..=500, width, Message::ImageWidthChanged))
            .push(text!("Width: {width} px").width(Fill).align_x(Center))
            .push(
                checkbox(filter_method == image::FilterMethod::Nearest)
                    .label("Use nearest interpolation")
                    .on_toggle(Message::ImageUseNearestToggled),
            )
            .align_x(Center)
    }

    fn scrollable(&self) -> Column<'_, Message> {
        Self::container("Scrollable")
            .push(
                "Iced supports scrollable content. Try it out! Find the \
                 button further below.",
            )
            .push(text("Tip: You can use the scrollbar to scroll down faster!").size(16))
            .push(space().height(4096))
            .push(
                text("You are halfway there!")
                    .width(Fill)
                    .size(30)
                    .align_x(Center),
            )
            .push(space().height(4096))
            .push(ferris(300, image::FilterMethod::Linear))
            .push(text("You made it!").width(Fill).size(50).align_x(Center))
    }

    fn text_input(&self) -> Column<'_, Message> {
        let value = &self.input_value;
        let is_secure = self.input_is_secure;
        let is_showing_icon = self.input_is_showing_icon;

        let mut text_input = text_input("Type something to continue...", value)
            .on_input(Message::InputChanged)
            .padding(10)
            .size(30);

        if is_showing_icon {
            text_input = text_input.icon(text_input::Icon {
                font: Font::default(),
                code_point: '🚀',
                size: Some(Pixels(28.0)),
                spacing: 10.0,
                side: text_input::Side::Right,
            });
        }

        Self::container("Text input")
            .push("Use a text input to ask for different kinds of information.")
            .push(text_input.secure(is_secure))
            .push(
                checkbox(is_secure)
                    .label("Enable password mode")
                    .on_toggle(Message::ToggleSecureInput),
            )
            .push(
                checkbox(is_showing_icon)
                    .label("Show icon")
                    .on_toggle(Message::ToggleTextInputIcon),
            )
            .push(
                "A text input produces a message every time it changes. It is \
                 very easy to keep track of its contents:",
            )
            .push(
                text(if value.is_empty() {
                    "You have not typed anything yet..."
                } else {
                    value
                })
                .width(Fill)
                .align_x(Center),
            )
    }

    fn debugger(&self) -> Column<'_, Message> {
        Self::container("Debugger")
            .push(
                "You can ask Iced to visually explain the layouting of the \
                 different elements comprising your UI!",
            )
            .push(
                "Give it a shot! Check the following checkbox to be able to \
                 see element boundaries.",
            )
            .push(
                checkbox(self.debug)
                    .label("Explain layout")
                    .on_toggle(Message::DebugToggled),
            )
            .push("Feel free to go back and take a look.")
    }

    fn end(&self) -> Column<'_, Message> {
        Self::container("You reached the end!")
            .push("This tour will be updated as more features are added.")
            .push("Make sure to keep an eye on it!")
    }

    fn rich_text(&self) -> Column<'_, Message> {
        let blue = color!(0x4090ff);
        let red = color!(0xff4040);
        let green = color!(0x40c040);

        let language = self.udhr_language as f32;

        Self::container("Rich text")
            .push("Iced supports bidirectional rich text.")
            .push(
                rich_text![
                    "You can ",
                    span("underline").underline(true),
                    " a span, ",
                    span("strike one through").strikethrough(true),
                    ", or ",
                    span("do both at once")
                        .color(blue)
                        .underline(true)
                        .strikethrough(true),
                    ". Decorations take the color of their span, like ",
                    span("red").color(red).underline(true),
                    " or ",
                    span("green").color(green).strikethrough(true),
                    "."
                ]
                .on_link_click(std::convert::identity),
            )
            .push(
                container(
                    column![
                        // A link with no explicit underline reveals one on
                        // hover, without relayouting.
                        rich_text![
                            span("Universal Declaration of Human Rights")
                                .size(16)
                                .color(color!(0x7777FF))
                                .link(Message::OpenDeclaration)
                        ]
                        .on_link_click(std::convert::identity),
                        scrollable(transition(
                            language,
                            move || { Animation::new(language).duration(Duration::from_secs(2)) },
                            |animation, now| {
                                let t = animation.interpolate_with(std::convert::identity, now);
                                let index = (t.round() as usize) % DECLARATIONS.len();
                                let alpha = (1.0 - 2.0 * (t - t.round()).abs()).clamp(0.0, 1.0);

                                declaration(index, alpha)
                            },
                        ))
                        .direction(scrollable::Direction::Vertical(
                            scrollable::Scrollbar::new()
                                .width(2)
                                .scroller_width(2)
                                .spacing(2),
                        ))
                        .height(Fill),
                    ]
                    .spacing(16),
                )
                .padding(20)
                .height(420)
                .style(container::bordered_box),
            )
    }

    fn container(title: &str) -> Column<'_, Message> {
        column![text(title).size(50)].spacing(20)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Welcome,
    Slider,
    RowsAndColumns,
    Text,
    RichText,
    Radio,
    Toggler,
    Image,
    Scrollable,
    TextInput,
    Debugger,
    End,
}

impl Screen {
    const ALL: &'static [Self] = &[
        Self::Welcome,
        Self::Slider,
        Self::RowsAndColumns,
        Self::Text,
        Self::RichText,
        Self::Radio,
        Self::Toggler,
        Self::Image,
        Self::Scrollable,
        Self::TextInput,
        Self::Debugger,
        Self::End,
    ];

    pub fn next(self) -> Option<Screen> {
        Self::ALL
            .get(
                Self::ALL
                    .iter()
                    .copied()
                    .position(|screen| screen == self)
                    .expect("Screen must exist")
                    + 1,
            )
            .copied()
    }

    pub fn previous(self) -> Option<Screen> {
        let position = Self::ALL
            .iter()
            .copied()
            .position(|screen| screen == self)
            .expect("Screen must exist");

        if position > 0 {
            Some(Self::ALL[position - 1])
        } else {
            None
        }
    }
}

fn ferris<'a>(width: u32, filter_method: image::FilterMethod) -> Container<'a, Message> {
    center_x(
        // This should go away once we unify resource loading on native
        // platforms
        if cfg!(target_arch = "wasm32") {
            image("tour/images/ferris.png")
        } else {
            image(concat!(env!("CARGO_MANIFEST_DIR"), "/images/ferris.png"))
        }
        .filter_method(filter_method)
        .width(width),
    )
}

fn padded_button<Message: Clone>(label: &str) -> Button<'_, Message> {
    button(text(label)).padding([12, 24])
}

fn color_slider<'a>(
    component: f32,
    update: impl Fn(f32) -> Color + 'a,
) -> Slider<'a, f64, Message> {
    slider(0.0..=1.0, f64::from(component), move |c| {
        Message::TextColorChanged(update(c as f32))
    })
    .step(0.01)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Elm,
    Ruby,
    Haskell,
    C,
    Other,
}

impl Language {
    fn all() -> [Language; 6] {
        [
            Language::C,
            Language::Elm,
            Language::Ruby,
            Language::Haskell,
            Language::Rust,
            Language::Other,
        ]
    }
}

impl From<Language> for String {
    fn from(language: Language) -> String {
        String::from(match language {
            Language::Rust => "Rust",
            Language::Elm => "Elm",
            Language::Ruby => "Ruby",
            Language::Haskell => "Haskell",
            Language::C => "C",
            Language::Other => "Other",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Row,
    Column,
}

/// A language name, followed by Articles 1, 2, and 3 of the Universal
/// Declaration of Human Rights in that language.
///
/// Article 1 is split into plain, underlined, plain, bold, and plain
/// chunks; Article 2 is two plain paragraphs; Article 3 is split into
/// plain, underlined, and plain chunks.
type Declaration = (
    &'static str,
    [&'static str; 5],
    [&'static str; 2],
    [&'static str; 3],
);

const DECLARATIONS: &[Declaration] = &[
    (
        "English",
        [
            "All human beings are born ",
            "free and equal in dignity and rights",
            ". They are endowed with ",
            "reason and conscience",
            " and should act towards one another in a spirit of brotherhood.",
        ],
        [
            "Everyone is entitled to all the rights and freedoms set forth in this Declaration, without distinction of any kind, such as race, colour, sex, language, religion, political or other opinion, national or social origin, property, birth or other status.",
            "Furthermore, no distinction shall be made on the basis of the political, jurisdictional or international status of the country or territory to which a person belongs, whether it be independent, trust, non-self-governing or under any other limitation of sovereignty.",
        ],
        [
            "Everyone has the right to ",
            "life, liberty and security of person",
            ".",
        ],
    ),
    (
        "العربية",
        [
            "يولد جميع الناس ",
            "أحرارًا متساوين في الكرامة والحقوق",
            ". وقد وهبوا ",
            "عقلاً وضميرًا",
            " وعليهم أن يعامل بعضهم بعضًا بروح الإخاء.",
        ],
        [
            "لكل إنسان حق التمتع بكافة الحقوق والحريات الواردة في هذا الإعلان، دون أي تمييز، كالتمييز بسبب العنصر أو اللون أو الجنس أو اللغة أو الدين أو الرأي السياسي أو أي رأي آخر، أو الأصل الوطني أو الإجتماعي أو الثروة أو الميلاد أو أي وضع آخر، دون أية تفرقة بين الرجال والنساء.",
            "وفضلاً عما تقدم فلن يكون هناك أي تمييز أساسه الوضع السياسي أو القانوني أو الدولي لبلد أو البقعة التي ينتمي إليها الفرد سواء كان هذا البلد أو تلك البقعة مستقلاً أو تحت الوصاية أو غير متمتع بالحكم الذاتي أو كانت سيادته خاضعة لأي قيد من القيود.",
        ],
        ["لكل فرد الحق في ", "الحياة والحرية وسلامة شخصه", "."],
    ),
    (
        "中文",
        [
            "人人生而自由，",
            "在尊严和权利上一律平等",
            "。他们赋有",
            "理性和良心",
            "，并应以兄弟关系的精神相对待。",
        ],
        [
            "人人有资格享有本宣言所载的一切权利和自由，不分种族、肤色、性别、语言、宗教、政治或其他见解、国籍或社会出身、财产、出生或其他身分等任何区别。",
            "并且不得因一人所属的国家或领土的政治的、行政的或者国际的地位之不同而有所区别，无论该领土是独立领土、托管领土、非自治领土或者处于其他任何主权受限制的情况之下。",
        ],
        ["人人有权享有", "生命、自由和人身安全", "。"],
    ),
    (
        "Ελληνικά",
        [
            "Όλοι οι άνθρωποι γεννιούνται ",
            "ελεύθεροι και ίσοι στην αξιοπρέπεια και τα δικαιώματα",
            ". Είναι προικισμένοι με ",
            "λογική και συνείδηση",
            ", και οφείλουν να συμπεριφέρονται μεταξύ τους με πνεύμα αδελφοσύνης.",
        ],
        [
            "Κάθε άνθρωπος δικαιούται να επικαλείται όλα τα δικαιώματα και όλες τις ελευθερίες που προκηρύσσει η παρούσα Διακήρυξη, χωρίς καμία απολύτως διάκριση, ειδικότερα ως προς τη φυλή, το χρώμα, το φύλο, τη γλώσσα, τις θρησκείες, τις πολιτικές ή οποιεσδήποτε άλλες πεποιθήσεις, την εθνική ή κοινωνική καταγωγή, την περιουσία, τη γέννηση ή οποιαδήποτε άλλη κατάσταση.",
            "Δεν θα μπορεί ακόμα να γίνεται καμία διάκριση εξαιτίας του πολιτικού, νομικού ή διεθνούς καθεστώτος της χώρας από την οποία προέρχεται κανείς, είτε πρόκειται για χώρα ή εδαφική περιοχή ανεξάρτητη, υπό κηδεμονία ή υπεξουσία, ή που βρίσκεται υπό οποιονδήποτε άλλον περιορισμό κυριαρχίας.",
        ],
        [
            "Κάθε άτομο έχει δικαίωμα στη ",
            "ζωή, την ελευθερία και την προσωπική του ασφάλεια",
            ".",
        ],
    ),
    (
        "עברית",
        [
            "כל בני אדם נולדו ",
            "בני חורין ושווים בערכם ובזכויותיהם",
            ". כולם חוננו ",
            "בתבונה ובמצפון",
            ", לפיכך חובה עליהם לנהוג איש ברעהו ברוח של אחוה.",
        ],
        [
            "כל אדם זכאי לזכויות ולחרויות שנקבעו בהכרזה זו ללא הפליה כלשהיא מטעמי גזע, צבע, מין, לשון, דת, דעה פוליטית או דעה בבעיות אחרות, בגלל מוצא לאומי או חברתי, קנין, לידה או מעמד אחר.",
            "גדולה מזו, לא יופלה אדם על פי מעמדה המדיני, על פי סמכותה או על פי מעמדה הבינלאומי של המדינה או הארץ שאליה הוא שייך, דין שהארץ היא עצמאית, ובין שהיא נתונה לנאמנות, בין שהיא נטולת שלטון עצמי ובין שריבונותה מוגבלת כל הגבלה אחרת.",
        ],
        ["כל אדם יש לו הזכות ", "לחיים, לחרות ולבטחון אישי", "."],
    ),
    (
        "हिन्दी",
        [
            "सभी मनुष्यों को गौरव और अधिकारों के मामले में ",
            "जन्मजात स्वतन्त्रता और समानता",
            " प्राप्त है । उन्हें ",
            "बुद्धि और अन्तरात्मा",
            " की देन प्राप्त है और परस्पर उन्हें भाईचारे के भाव से बर्ताव करना चाहिए ।",
        ],
        [
            "सभी को इस घोषणा में सन्निहित सभी अधिकारों और आज़ादियों को प्राप्त करने का हक़ है और इस मामले में जाति, वर्ण, लिंग, भाषा, धर्म, राजनीति या अन्य विचार-प्रणाली, किसी देश या समाज विशेष में जन्म, सम्पत्ति या किसी प्रकार की अन्य मर्यादा आदि के कारण भेदभाव का विचार न किया जाएगा ।",
            "इसके अतिरिक्त, चाहे कोई देश या प्रदेश स्वतन्त्र हो, संरक्षित हो, या स्त्रशासन रहित हो या परिमित प्रभुसत्ता वाला हो, उस देश या प्रदेश की राजनैतिक, क्षेत्रीय या अन्तर्राष्ट्रीय स्थिति के आधार पर वहां के निवासियों के प्रति कोई फ़रक़ न रखा जाएगा ।",
        ],
        [
            "प्रत्येक व्यक्ति को ",
            "जीवन, स्वाधीनता और वैयक्तिक सुरक्षा",
            " का अधिकार है ।",
        ],
    ),
    (
        "日本語",
        [
            "すべての人間は、",
            "生まれながらにして自由であり、かつ、尊厳と権利とについて平等である",
            "。人間は、",
            "理性と良心",
            "とを授けられており、互いに同胞の精神をもって行動しなければならない。",
        ],
        [
            "すべて人は、人種、皮膚の色、性、言語、宗教、政治上その他の意見、国民的もしくは社会的出身、財産、門地その他の地位又はこれに類するいかなる事由による差別をも受けることなく、この宣言に掲げるすべての権利と自由とを享有することができる。",
            "さらに、個人の属する国又は地域が独立国であると、信託統治地域であると、非自治地域であると、又は他のなんらかの主権制限の下にあるを問わず、その国又は地域の政治上、管轄上又は国際上の地位に基づくいかなる差別もしてはならない。",
        ],
        [
            "すべての人は、",
            "生命、自由及び身体の安全",
            "に対する権利を有する。",
        ],
    ),
    (
        "한국어",
        [
            "모든 인간은 태어날 때부터 ",
            "자유로우며 그 존엄과 권리에 있어 동등하다",
            ". 인간은 천부적으로 ",
            "이성과 양심",
            "을 부여받았으며 서로 형제애의 정신으로 행동하여야 한다.",
        ],
        [
            "모든 사람은 인종, 피부색, 성, 언어, 종교, 정치적 또는 기타의 견해, 민족적 또는 사회적 출신, 재산, 출생 또는 기타의 신분과 같은 어떠한 종류의 차별이 없이, 이 선언에 규정된 모든 권리와 자유를 향유할 자격이 있다.",
            "더 나아가 개인이 속한 국가 또는 영토가 독립국, 신탁통치지역, 비자치지역이거나 또는 주권에 대한 여타의 제약을 받느냐에 관계없이, 그 국가 또는 영토의 정치적, 법적 또는 국제적 지위에 근거하여 차별이 있어서는 아니된다.",
        ],
        [
            "모든 사람은 ",
            "생명과 신체의 자유와 안전",
            "에 대한 권리를 가진다.",
        ],
    ),
    (
        "فارسی",
        [
            "تمام افراد بشر ",
            "آزاد بدنیا میایند و از لحاظ حیثیت و حقوق با هم برابرند",
            ". همه دارای ",
            "عقل و وجدان",
            " میباشند و باید نسبت بیکدیگر با روح برادری رفتار کنند.",
        ],
        [
            "هر کس میتواند بدون هیچگونه تمایز مخصوصا از حیث نژاد، رنگ، جنس، زبان، مذهب، عقیدهٔ سیاسی یا هر عقیده دیگر و همچنین ملیت، وضع اجتماعی، ثروت، ولادت یا هر موقعیت دیگر، از تمام حقوق و کلیهٔ آزادی\u{200c}هائیکه در اعلامیه ذکر حاضر شده است، بهره\u{200c}مند گردد.",
            "بعلاوه هیچ تبعیضی بعمل نخواهد آمد که مبتنی بر وضع سیاسی، اداری و قضائی یا بین\u{200c}المللی کشور یا سرزمینی باشد که شخص بآن تعلق دارد، خواه این کشور مستقل، تحت قیومیت یا غیر خودمختار بوده یا حاکمیت آن بشکلی محدود شده باشد.",
        ],
        ["هر کس حق ", "زندگی، آزادی و امنیت شخصی", " دارد."],
    ),
    (
        "Русский",
        [
            "Все люди рождаются ",
            "свободными и равными в своем достоинстве и правах",
            ". Они наделены ",
            "разумом и совестью",
            " и должны поступать в отношении друг друга в духе братства.",
        ],
        [
            "Каждый человек должен обладать всеми правами и всеми свободами, провозглашенными настоящей Декларацией, без какого бы то ни было различия, как-то в отношении расы, цвета кожи, пола, языка, религии, политических или иных убеждений, национального или социального происхождения, имущественного, сословного или иного положения.",
            "Кроме того, не должно проводиться никакого различия на основе политического, правового или международного статуса страны или территории, к которой человек принадлежит, независимо от того, является ли эта территория независимой, подопечной, несамоуправляющейся или как-либо иначе ограниченной в своем суверенитете.",
        ],
        [
            "Каждый человек имеет право на ",
            "жизнь, на свободу и на личную неприкосновенность",
            ".",
        ],
    ),
    (
        "Español",
        [
            "Todos los seres humanos nacen ",
            "libres e iguales en dignidad y derechos",
            " y, dotados como están de ",
            "razón y conciencia",
            ", deben comportarse fraternalmente los unos con los otros.",
        ],
        [
            "Toda persona tiene los derechos y libertades proclamados en esta Declaración, sin distinción alguna de raza, color, sexo, idioma, religión, opinión política o de cualquier otra índole, origen nacional o social, posición económica, nacimiento o cualquier otra condición.",
            "Además, no se hará distinción alguna fundada en la condición política, jurídica o internacional del país o territorio de cuya jurisdicción dependa una persona, tanto si se trata de un país independiente, como de un territorio bajo administración fiduciaria, no autónomo o sometido a cualquier otra limitación de soberanía.",
        ],
        [
            "Todo individuo tiene derecho a ",
            "la vida, a la libertad y a la seguridad de su persona",
            ".",
        ],
    ),
];

fn declaration<'a>(language: usize, alpha: f32) -> Element<'a, Message> {
    let (name, first, second, third) = &DECLARATIONS[language];
    let [a, b, c, d, e] = first;
    let [f, g] = second;
    let [h, i, j] = third;

    let blue = color!(0x4090ff).scale_alpha(alpha);
    let green = color!(0x40c040).scale_alpha(alpha);
    let bold = Font {
        weight: font::Weight::Bold,
        ..Font::DEFAULT
    };
    let body = move |theme: &Theme| iced::widget::text::Style {
        color: Some(theme.palette().background.base.text.scale_alpha(alpha)),
    };
    let align = if matches!(name.chars().next(), Some('\u{0590}'..='\u{08FF}')) {
        iced::Right
    } else {
        iced::Left
    };

    column![
        text(*name)
            .size(13)
            .width(Fill)
            .align_x(align)
            .color(color!(0x888888).scale_alpha(alpha)),
        rich_text![
            span(*a),
            span(*b).underline(true).color(blue),
            span(*c),
            span(*d).font(bold),
            span(*e),
        ]
        .width(Fill)
        .align_x(align)
        .style(body)
        .on_link_click(std::convert::identity),
        text(*f).width(Fill).align_x(align).style(body),
        text(*g).width(Fill).align_x(align).style(body),
        rich_text![span(*h), span(*i).underline(true).color(green), span(*j),]
            .width(Fill)
            .align_x(align)
            .style(body)
            .on_link_click(std::convert::identity),
    ]
    .spacing(12)
    .into()
}

impl Default for Tour {
    fn default() -> Self {
        Self {
            screen: Screen::Welcome,
            slider: 50,
            layout: Layout::Row,
            spacing: 20,
            text_size: 30,
            text_color: Color::BLACK,
            language: None,
            toggler: false,
            image_width: 300,
            image_filter_method: image::FilterMethod::Linear,
            input_value: String::new(),
            input_is_secure: false,
            input_is_showing_icon: false,
            debug: false,
            udhr_language: 0,
        }
    }
}
