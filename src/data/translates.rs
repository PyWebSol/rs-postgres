pub enum Language {
    English,
    Russian,
}

pub struct Translator {
    pub language: Language,
}

impl Translator {
    pub fn new(language: Language) -> Self {
        Self { language }
    }
}
