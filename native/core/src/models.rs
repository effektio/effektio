mod color;
mod faq;
mod news;
mod tag;
mod tasks;

pub use color::Color;
pub use faq::Faq;
pub use news::News;
pub use tag::Tag;
pub use tasks::{Task, TaskList};

#[cfg(feature = "with-mocks")]
pub mod mocks {
    pub use super::color::mocks::ColorFaker;
    pub use super::faq::gen_mocks as gen_mock_faqs;
    pub use super::news::gen_mocks as gen_mock_news;
}
