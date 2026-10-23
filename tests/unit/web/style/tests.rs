//! The console stylesheet is assembled, so the assembly itself needs a guard.

/// The order the slices are joined in is not cosmetic.
///
/// CSS resolves by source order, so a slice joined in the wrong place does not
/// fail: the rules simply stop winning the cascade they were written to win, and
/// the page renders *nearly* right. Markers below are selectors that occur in
/// exactly one slice, so their positions in the joined sheet pin the order.
#[test]
fn the_slices_are_joined_in_the_order_the_cascade_depends_on() {
    let css = super::css();
    let markers = [
        ".skip-link {",
        ".status-dot {",
        ".dashboard-grid {",
        ".aws-top-bar {",
    ];
    let mut previous = 0;
    for marker in markers {
        // `find` after `contains` so a missing marker produces an assertion
        // naming it, rather than an unwrap that says only "None".
        assert!(
            css.contains(marker),
            "{marker} is missing from the joined stylesheet"
        );
        let at = css.find(marker).unwrap_or_default();
        assert!(
            at >= previous,
            "{marker} appears before the slice that should precede it"
        );
        previous = at;
    }
}

/// The sheet is the four slices, in order, with nothing added.
///
/// `concat` inserts no separator, so as written this is a property rather than a
/// risk being guarded. It is still the only test that would notice a `.join()`
/// replacing it, a slice attached twice, or a fifth part appended somewhere the
/// module docs do not mention — and a stylesheet has no other check. Nothing
/// renders it: a rule that went missing here is invisible to every other test
/// and to `cargo build`, and shows up only as a page that looks slightly wrong.
#[test]
fn the_joined_sheet_is_exactly_its_parts_with_nothing_between_them() {
    let expected = [
        super::foundation::RULES,
        super::controls::RULES,
        super::screens::RULES,
        super::aws::RULES,
    ]
    .concat();
    assert_eq!(super::css(), expected);
}

/// Assembling is done once, and every caller sees the same bytes.
///
/// Every page renders the sheet into its own markup, so rebuilding the join per
/// call would allocate forty-odd kilobytes per response and hand the CSP a
/// different allocation each time. Sharing one `String` keeps both costs off the
/// render path.
#[test]
fn the_sheet_is_built_once_and_shared() {
    assert!(std::ptr::eq(super::css(), super::css()));
}
