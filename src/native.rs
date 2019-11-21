pub use iced_winit::{
    Align, Background, Color, Command, Font, HorizontalAlignment, Length,
    VerticalAlignment,
};

pub mod widget {
    pub mod button {
        pub type Button<'a, Message> =
            iced_winit::Button<'a, Message, iced_wgpu::Renderer>;

        pub use iced_winit::button::State;
    }

    pub mod scrollable {
        pub type Scrollable<'a, Message> =
            iced_winit::Scrollable<'a, Message, iced_wgpu::Renderer>;

        pub use iced_winit::scrollable::State;
    }

    pub mod text_input {
        pub use iced_winit::text_input::{State, TextInput};
    }

    pub mod slider {
        pub use iced_winit::slider::{Slider, State};
    }

    pub use iced_winit::{Checkbox, Image, Radio, Text};

    #[doc(no_inline)]
    pub use {
        button::Button, scrollable::Scrollable, slider::Slider,
        text_input::TextInput,
    };

    pub type Column<'a, Message> =
        iced_winit::Column<'a, Message, iced_wgpu::Renderer>;

    pub type Row<'a, Message> =
        iced_winit::Row<'a, Message, iced_wgpu::Renderer>;

    pub type Container<'a, Message> =
        iced_winit::Container<'a, Message, iced_wgpu::Renderer>;
}

#[doc(no_inline)]
pub use widget::*;

pub type Element<'a, Message> =
    iced_winit::Element<'a, Message, iced_wgpu::Renderer>;
