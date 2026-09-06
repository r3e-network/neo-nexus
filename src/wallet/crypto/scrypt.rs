/// Upper bounds shared by wallet validation and local NEP-2 decryption.
///
/// The canonical Neo profile (16384, 8, 8) uses roughly 16 MiB. Accepting the
/// bounded NEP-6 fields keeps genuine non-default wallets interoperable without
/// allowing a small JSON file to request unbounded memory or CPU.
pub(crate) fn valid_scrypt_parameters(n: u64, r: u64, p: u64) -> bool {
    const MIN_MEMORY_BYTES: u64 = 16 * 1024 * 1024;
    const MIN_WORK: u64 = 1_048_576;
    const MAX_MEMORY_BYTES: u64 = 128 * 1024 * 1024;
    const MAX_WORK: u64 = 16_777_216;

    (1_024..=1_048_576).contains(&n)
        && n.is_power_of_two()
        && (1..=32).contains(&r)
        && (1..=16).contains(&p)
        && n.checked_mul(r)
            .and_then(|blocks| blocks.checked_mul(128))
            .is_some_and(|bytes| (MIN_MEMORY_BYTES..=MAX_MEMORY_BYTES).contains(&bytes))
        && n.checked_mul(r)
            .and_then(|work| work.checked_mul(p))
            .is_some_and(|work| (MIN_WORK..=MAX_WORK).contains(&work))
}
