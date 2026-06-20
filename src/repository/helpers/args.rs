const ARG_SEPARATOR: char = '\u{1f}';

pub(in crate::repository) fn encode_args(args: &[String]) -> String {
    args.join(&ARG_SEPARATOR.to_string())
}

pub(in crate::repository) fn decode_args(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        Vec::new()
    } else {
        raw.split(ARG_SEPARATOR).map(ToString::to_string).collect()
    }
}
