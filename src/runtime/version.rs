use std::cmp::Ordering;

pub(super) fn compare_versions(left: &str, right: &str) -> Ordering {
    let left_numbers = version_numbers(left);
    let right_numbers = version_numbers(right);
    if left_numbers.is_empty() || right_numbers.is_empty() {
        return left.cmp(right);
    }

    let length = left_numbers.len().max(right_numbers.len());
    for index in 0..length {
        let left_value = left_numbers.get(index).copied().unwrap_or(0);
        let right_value = right_numbers.get(index).copied().unwrap_or(0);
        match left_value.cmp(&right_value) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
    }

    left.cmp(right)
}

fn version_numbers(version: &str) -> Vec<u64> {
    version
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u64>().ok())
        .collect()
}
