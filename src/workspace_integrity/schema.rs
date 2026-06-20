mod indexes;
mod tables;

pub(super) use indexes::REQUIRED_INDEXES;
pub(super) use tables::required_tables;

#[derive(Debug, Clone, Copy)]
pub(super) struct RequiredTable {
    pub(super) name: &'static str,
    pub(super) columns: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RequiredIndex {
    pub(super) table: &'static str,
    pub(super) name: &'static str,
}
