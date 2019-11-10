pub trait Application {
    type Message : std::fmt::Debug;
    type Renderer : crate::renderer::Windowed + crate::column::Renderer;

    fn title(&self) -> String;
    fn update(&mut self, message: Self::Message);
    fn view(&mut self) -> crate::Element<Self::Message, Self::Renderer>;
}
