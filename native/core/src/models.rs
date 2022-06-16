use serde::{Serialize, Deserialize};

mod color;
mod faq;
mod news;
mod tag;

pub use color::Color;
pub use faq::Faq;
pub use news::News;
pub use tag::Tag;


#[cfg(feature = "with-mocks")]
pub mod mocks {
    pub use super::color::mocks::ColorFaker;
    pub use super::faq::gen_mocks as gen_mock_faqs;
    pub use super::news::gen_mocks as gen_mock_news;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EffektioModel {
    News(News),
//    TextMessage(),
}

impl EffektioModel {
    pub fn indizes(&self) -> Vec<String> {
        match self {
            EffektioModel::News(_) => vec!["type-news".to_string(), "section-news".to_string()],
        }
    }
}