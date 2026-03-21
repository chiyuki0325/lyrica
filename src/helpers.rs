pub(crate) trait StringVecExt {
    fn contains(&self, needle: &str) -> bool;
    fn position_of(&self, needle: &str) -> Option<usize>;
}

// Rust prefer use "Ext" instead of "Helper"

impl StringVecExt for Vec<String> {
    fn contains(&self, needle: &str) -> bool {
        self.iter().any(|it| it == needle)
    }

    fn position_of(&self, needle: &str) -> Option<usize> {
        self.iter().position(|it| it == needle)
    }
}
