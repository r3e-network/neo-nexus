use anyhow::Result;

/// Fails when a write affected no rows, signalling that the targeted record did
/// not exist. Centralizes the `"<entity> {id} was not found"` error used by
/// every delete, update, and mark operation so the phrasing stays consistent.
pub(in crate::repository) fn ensure_affected_rows(
    affected: usize,
    entity: &str,
    id: &str,
) -> Result<()> {
    if affected == 0 {
        anyhow::bail!("{entity} {id} was not found");
    }
    Ok(())
}
