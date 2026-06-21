pub(super) fn clamp_page(current: usize, item_count: usize, page_size: usize) -> usize {
    current.min(page_count(item_count, page_size) - 1)
}

pub(super) fn page_count(item_count: usize, page_size: usize) -> usize {
    item_count.div_ceil(page_size).max(1)
}

#[cfg(test)]
#[path = "../../tests/unit/app/paging/tests.rs"]
mod tests;
