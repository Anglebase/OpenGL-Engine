mod app;
pub mod log;

pub use app::*;
pub use log::*;

pub use gom::{id, Registry};
/// 窗口实例类型
pub type Window = glfw::PWindow;