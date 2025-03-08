#![allow(dead_code)]
use crate::core::Element;
use crate::core::overlay;

use ouroboros::self_referencing;

#[self_referencing(pub_extras)]
pub struct Cache<'a, Message: 'a, Theme: 'a, Renderer: 'a> {
    pub element: Element<'a, Message, Theme, Renderer>,

    #[borrows(mut element)]
    #[covariant]
    overlay: Option<overlay::Element<'this, Message, Theme, Renderer>>,
}
