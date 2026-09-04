use std::cmp::Ordering;

pub(super) fn compare_versions(left: &str, right: &str) -> Ordering {
    match (parse(left), parse(right)) {
        (Some((left_core, left_pre)), Some((right_core, right_pre))) => {
            for index in 0..left_core.len().max(right_core.len()) {
                let order = left_core
                    .get(index)
                    .unwrap_or(&0)
                    .cmp(right_core.get(index).unwrap_or(&0));
                if order != Ordering::Equal {
                    return order;
                }
            }
            match (left_pre, right_pre) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(left), Some(right)) => compare_prerelease(left, right),
            }
        }
        _ => left.cmp(right),
    }
}

fn parse(version: &str) -> Option<(Vec<u64>, Option<&str>)> {
    let version = version.trim().trim_start_matches('v').split('+').next()?;
    let (core, pre) = version
        .split_once('-')
        .map_or((version, None), |(core, pre)| (core, Some(pre)));
    let numbers = core
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    (!numbers.is_empty()).then_some((numbers, pre))
}

fn compare_prerelease(left: &str, right: &str) -> Ordering {
    let mut left = left.split('.');
    let mut right = right.split('.');
    loop {
        match (left.next(), right.next()) {
            (Some(left), Some(right)) => {
                let order = match (left.parse::<u64>(), right.parse::<u64>()) {
                    (Ok(left), Ok(right)) => left.cmp(&right),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => left.cmp(right),
                };
                if order != Ordering::Equal {
                    return order;
                }
            }
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn release_versions_do_not_upgrade_back_to_prereleases() {
        assert_eq!(compare_versions("v3.8.0", "3.8.0-rc.9"), Ordering::Greater);
        assert_eq!(
            compare_versions("3.8.0-rc.10", "3.8.0-rc.2"),
            Ordering::Greater
        );
        assert_eq!(
            compare_versions("3.8.0-alpha", "3.8.0-alpha.1"),
            Ordering::Less
        );
        assert_eq!(
            compare_versions("3.8.0+sha.2", "v3.8.0+sha.1"),
            Ordering::Equal
        );
        assert_eq!(compare_versions("v3.10.0", "v3.9.9"), Ordering::Greater);
        assert_eq!(compare_versions("0.110.0", "0.99.9"), Ordering::Greater);
    }
}
