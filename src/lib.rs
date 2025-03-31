use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

mod languages;
mod preprocessor;

pub struct LangTabsPreprocessor;

impl Default for LangTabsPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl LangTabsPreprocessor {
    pub fn new() -> Self {
        LangTabsPreprocessor
    }
}

impl Preprocessor for LangTabsPreprocessor {
    fn name(&self) -> &str {
        "langtabs"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                preprocessor::process_chapter(chapter);
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}
