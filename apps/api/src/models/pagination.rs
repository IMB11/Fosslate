#[derive(Debug, Clone, Copy)]
pub struct KeysetPage {
    pub after_id: Option<i64>,
    pub limit: i64,
}

impl KeysetPage {
    pub fn new(after_id: Option<i64>, limit: Option<i64>) -> Self {
        Self {
            after_id,
            limit: limit.unwrap_or(100).clamp(1, 500),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyset_page_clamps_limit() {
        assert_eq!(KeysetPage::new(None, None).limit, 100);
        assert_eq!(KeysetPage::new(None, Some(0)).limit, 1);
        assert_eq!(KeysetPage::new(None, Some(900)).limit, 500);
    }
}
