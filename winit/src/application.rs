pub trait Application {
    type Renderer : crate::renderer::Windowed;
    type Message : std::fmt::Debug + 'static;

    fn title(&self) -> String;
    fn update(&mut self, message: Self::Message);
    fn view(&mut self) -> crate::Element<Self::Message, Self::Renderer>;
    fn style(&self) -> crate::renderer::Style { crate::renderer::Style::default() }
}
