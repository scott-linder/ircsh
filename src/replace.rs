//! Iterators which allows items to be replaced.
//!
//! The items are not forwarded back to the underlying iterator, but are held
//! internally.

/// Allows only one item to be replaced at a time.
///
/// It is not an error to replace a character on-top of another.
pub struct ReplaceOne<I> where I: Iterator {
    iter: I,
    replaced: Option<I::Item>,
}

impl<I: Iterator> ReplaceOne<I> {
    pub fn new(iter: I) -> ReplaceOne<I> {
        ReplaceOne {
            iter: iter,
            replaced: None,
        }
    }

    pub fn replace(&mut self, item: I::Item) {
        self.replaced = Some(item);
    }
}

impl<I: Iterator> Iterator for ReplaceOne<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        match self.replaced.take() {
            i @ Some(_) => i,
            None => self.iter.next(),
        }
    }
}
