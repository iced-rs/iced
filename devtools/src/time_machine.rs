use crate::Program;

#[cfg(feature = "time-travel")]
pub struct TimeMachine<P>
where
    P: Program,
{
    state: Option<P::State>,
    messages: Vec<P::Message>,
}

#[cfg(feature = "time-travel")]
impl<P> TimeMachine<P>
where
    P: Program,
{
    pub fn new() -> Self {
        Self {
            state: None,
            messages: Vec::new(),
        }
    }

    pub fn is_rewinding(&self) -> bool {
        self.state.is_some()
    }

    pub fn push(&mut self, message: &P::Message) {
        self.messages.push(message.clone());
    }

    pub fn rewind(&mut self, program: &P, message: usize) {
        let (mut state, _) = program.boot();

        if message < self.messages.len() {
            // TODO: Run concurrently (?)
            for message in &self.messages[0..message] {
                let _ = program.update(&mut state, message.clone());
            }
        }

        self.state = Some(state);
        crate::debug::disable();
    }

    pub fn go_to_present(&mut self) {
        self.state = None;
        crate::debug::enable();
    }

    pub fn state(&self) -> Option<&P::State> {
        self.state.as_ref()
    }
}

#[cfg(not(feature = "time-travel"))]
pub struct TimeMachine<P>
where
    P: Program,
{
    _program: std::marker::PhantomData<P>,
}

#[cfg(not(feature = "time-travel"))]
impl<P> TimeMachine<P>
where
    P: Program,
{
    pub fn new() -> Self {
        Self {
            _program: std::marker::PhantomData,
        }
    }

    pub fn is_rewinding(&self) -> bool {
        false
    }

    pub fn push(&mut self, _message: &P::Message) {}

    pub fn rewind(&mut self, _program: &P, _message: usize) {}

    pub fn go_to_present(&mut self) {}

    pub fn state(&self) -> Option<&P::State> {
        None
    }
}
