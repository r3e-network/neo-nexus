pub(super) fn clamp_page(current: usize, item_count: usize, page_size: usize) -> usize {
    current.min(page_count(item_count, page_size) - 1)
}

pub(super) fn page_count(item_count: usize, page_size: usize) -> usize {
    item_count.div_ceil(page_size).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_count_never_returns_zero() {
        assert_eq!(page_count(0, 7), 1);
        assert_eq!(page_count(1, 7), 1);
        assert_eq!(page_count(8, 7), 2);
    }

    #[test]
    fn clamp_page_keeps_page_in_range() {
        assert_eq!(clamp_page(5, 0, 7), 0);
        assert_eq!(clamp_page(5, 8, 7), 1);
        assert_eq!(clamp_page(0, 8, 7), 0);
    }
}
