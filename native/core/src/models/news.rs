use super::{Color, Tag};
use serde::{Serialize, Deserialize};

#[cfg(feature = "with-mocks")]
use super::mocks::ColorFaker;
#[cfg(feature = "with-mocks")]
use fake::{faker::lorem::en::Sentence, Dummy, Fake, Faker};

#[cfg(feature = "with-mocks")]
pub(crate) mod mocks {
    use fake::Dummy;
    use rand::prelude::*;
    use rand::Rng;

    pub struct RandomImage;
    pub static IMAGE_001: &[u8] = include_bytes!("./mocks/images/01.jpg").as_slice();
    pub static IMAGE_002: &[u8] = include_bytes!("./mocks/images/02.jpg").as_slice();
    pub static IMAGE_003: &[u8] = include_bytes!("./mocks/images/03.jpg").as_slice();
    pub static IMAGE_004: &[u8] = include_bytes!("./mocks/images/04.jpg").as_slice();
    pub static IMAGE_005: &[u8] = include_bytes!("./mocks/images/05.jpg").as_slice();
    pub static IMAGE_006: &[u8] = include_bytes!("./mocks/images/06.jpg").as_slice();
    pub static IMAGE_007: &[u8] = include_bytes!("./mocks/images/07.jpg").as_slice();

    impl Dummy<RandomImage> for Vec<u8> {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &RandomImage, rng: &mut R) -> Self {
            vec![
                IMAGE_001, IMAGE_002, IMAGE_003, IMAGE_004, IMAGE_005, IMAGE_006, IMAGE_007,
            ]
            .choose(rng)
            .unwrap()
            .to_vec()
        }
    }
}

#[cfg_attr(feature = "with-mocks", derive(Dummy))]
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct News {
    #[cfg_attr(feature = "with-mocks", dummy(faker = "Sentence(3..24)"))]
    pub(crate) text: Option<String>,
    pub(crate) tags: Vec<Tag>,
    #[cfg_attr(feature = "with-mocks", dummy(faker = "0..1024"))]
    pub(crate) likes_count: u64,
    #[cfg_attr(feature = "with-mocks", dummy(faker = "0..324"))]
    pub(crate) comments_count: u64,
    #[cfg_attr(feature = "with-mocks", dummy(faker = "ColorFaker"))]
    pub(crate) bg_color: Option<Color>,
    #[cfg_attr(feature = "with-mocks", dummy(faker = "ColorFaker"))]
    pub(crate) fg_color: Option<Color>,
    #[cfg_attr(feature = "with-mocks", dummy(faker = "mocks::RandomImage"))]
    pub(crate) image: Option<Vec<u8>>,
}

impl News {
    pub fn text(&self) -> Option<String> {
        self.text.clone()
    }
    pub fn tags(&self) -> Vec<Tag> {
        self.tags.clone()
    }
    pub fn likes_count(&self) -> u64 {
        self.likes_count
    }
    pub fn comments_count(&self) -> u64 {
        self.comments_count
    }
    pub fn bg_color(&self) -> Option<Color> {
        self.bg_color.clone()
    }
    pub fn fg_color(&self) -> Option<Color> {
        self.fg_color.clone()
    }
    pub fn image(&self) -> Option<Vec<u8>> {
        self.image.clone()
    }
}

#[cfg(feature = "with-mocks")]
pub fn gen_mocks() -> Vec<News> {
    vec![
        News {
            text: Some("Our transport is on its way to Poland, to refugee camps".to_string()),
            likes_count: 23,
            comments_count: 2,
            image: Some(mocks::IMAGE_001.to_vec()),
            fg_color: Some(Color::from_rgb_u8(255, 255, 255)),
            bg_color: Some(Color::from_rgb_u8(0, 0, 0)),
            ..Default::default()
        },
        News {
            text: Some("What is currently needed to help Ukrainian refugees ".to_string()),
            likes_count: 102,
            comments_count: 14,
            image: Some(mocks::IMAGE_002.to_vec()),
            fg_color: Some(Color::from_rgb_u8(255, 255, 255)),
            bg_color: Some(Color::from_rgb_u8(0, 0, 0)),
            ..Default::default()
        },
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
    ]
}
