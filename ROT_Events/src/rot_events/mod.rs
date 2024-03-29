#[allow(unused_imports)]
#[allow(non_camel_case_types)]
pub mod commom;
pub mod keyboard;
pub mod mouse;

pub use commom as event;
pub use keyboard as KeyboardInput;
pub use mouse as MouseInput;
