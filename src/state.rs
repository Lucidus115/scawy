use crate::Context;
pub struct AppState(Vec<Box<dyn State>>);

pub mod game;

impl AppState {
    pub fn new(default_state: Box<dyn State>) -> Self {
        Self(vec![default_state])
    }

    pub fn push(&mut self, state: Box<dyn State>) {
        self.0.push(state)
    }

    pub fn pop(&mut self) -> Box<dyn State> {
        self.0.pop().expect("Last state should never be popped off")
    }

    #[allow(clippy::borrowed_box)]
    pub fn peek(&mut self) -> &mut Box<dyn State> {
        self.0
            .last_mut()
            .expect("There should always be at least one state in the stack")
    }
}

pub trait State {
    fn update(&mut self, ctx: &mut Context);

    fn draw(&mut self, ctx: &mut Context, screen: &mut [u8]);
}
