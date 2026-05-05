pub struct Report<'a> {
    pub title: &'a str,
    pub rows: usize,
}

impl Report<'_> {
    pub fn is_empty(&self) -> bool {
        self.rows == 0
    }
}
