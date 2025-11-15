pub struct Bp {
    pub left: u8,
    pub right: u8,
}

impl From<(u8, u8)> for Bp {
    fn from(value: (u8, u8)) -> Self {
        let (left, right) = value;
        Self { left, right }
    }
}
