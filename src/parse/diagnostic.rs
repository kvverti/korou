//! Types relating to parser diagnostics.

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Diagnostics {
    errors: Vec<String>,
}

impl Diagnostics {
    pub const fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    pub fn error<'s>(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }

    pub fn combine(&mut self, others: Self) {
        self.errors.extend(others.errors);
    }
}

impl IntoIterator for Diagnostics {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}
