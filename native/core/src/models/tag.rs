use super::Color;
use serde::{Serialize, Deserialize};

#[cfg(feature = "with-mocks")]
use super::mocks::ColorFaker;
#[cfg(feature = "with-mocks")]
use fake::{faker::lorem::en::Word, Dummy, Fake};

#[cfg_attr(feature = "with-mocks", derive(Dummy))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tag {
    #[cfg_attr(feature = "with-mocks", dummy(faker = "Word()"))]
    pub(crate) title: String,
    #[cfg_attr(feature = "with-mocks", dummy(faker = "ColorFaker"))]
    pub(crate) color: Option<Color>,
}

impl Tag {
    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn hash_tag(&self) -> String {
        self.title.to_lowercase()
    }

    pub fn color(&self) -> Option<Color> {
        self.color.clone()
    }
}
