/*
 * This trait is an interface for any system which can render the output of 
 * the Cpu.
 */
pub trait Screen {
    fn print_console(&self, character: char);
    fn print_program(&self);
    fn print_memory(&self);
}

pub struct BasicScreen;

impl BasicScreen {
    pub fn new() -> BasicScreen {
        BasicScreen
    }
}

impl Screen for BasicScreen {
    fn print_console(&self, character: char) {
        print!("{}", character);
    }
    fn print_program(&self) {}
    fn print_memory(&self) {}
}
