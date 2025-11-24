#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReturnStatus {
    Always,
    Never,
    Sometimes,
}

impl ReturnStatus {
    pub fn intersect(self, other: Self) -> Self {
        match (self, other) {
            (ReturnStatus::Always, ReturnStatus::Always) => ReturnStatus::Always,
            (ReturnStatus::Never, ReturnStatus::Never) => ReturnStatus::Never,
            _ => ReturnStatus::Sometimes,
        }
    }

    pub fn then(self, other: Self) -> Self {
        match self {
            ReturnStatus::Always => ReturnStatus::Always,
            _ => other,
        }
    }
}
