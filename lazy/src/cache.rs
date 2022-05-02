use iced_native::overlay;
use iced_native::Element;

use ouroboros::self_referencing;

#[self_referencing(pub_extras)]
pub struct Cache<'a, Message: 'a, Renderer: 'a> {
    pub element: Element<'a, Message, Renderer>,

    #[borrows(mut element)]
    #[covariant]
    pub overlay: Option<overlay::Element<'this, Message, Renderer>>,
}
