//! Ratatelier is a terminal-native studio for cell artwork, animation, and
//! reusable Ratatui component mockups.

pub mod app;
pub mod export;
pub mod geometry;
pub mod model;
pub mod storage;
pub mod ui;
pub mod viewer;

pub use app::App;
pub use model::Project;
