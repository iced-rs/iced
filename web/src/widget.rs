use crate::Bus;
use dodrio::bumpalo;

pub mod button;
pub mod slider;
pub mod text;

mod checkbox;
mod column;
mod image;
mod radio;
mod row;

#[doc(no_inline)]
pub use button::Button;

#[doc(no_inline)]
pub use slider::Slider;

#[doc(no_inline)]
pub use text::Text;

pub use checkbox::Checkbox;
pub use column::Column;
pub use image::Image;
pub use radio::Radio;
pub use row::Row;

pub trait Widget<Message> {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _bus: &Bus<Message>,
    ) -> dodrio::Node<'b>;
}
