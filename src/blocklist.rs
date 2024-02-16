pub struct Blocklist {
}

impl Blocklist {
    pub fn new() -> Self {
        Blocklist {}
    }

    pub fn contains(&self, _domain: &str) -> bool {
        false
    }
}