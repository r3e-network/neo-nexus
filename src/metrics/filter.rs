use super::{MissingProcessMetric, NodeProcessMetrics};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStateFilter {
    Observed,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProcessFilter {
    pub state: Option<ProcessStateFilter>,
    pub high_cpu: bool,
    pub high_memory: bool,
    pub query: String,
}

impl ProcessFilter {
    pub fn new(
        state: Option<ProcessStateFilter>,
        high_cpu: bool,
        high_memory: bool,
        query: impl Into<String>,
    ) -> Self {
        Self {
            state,
            high_cpu,
            high_memory,
            query: query.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessRow {
    Observed(NodeProcessMetrics),
    Missing(MissingProcessMetric),
}

impl ProcessRow {
    pub fn node_id(&self) -> &str {
        match self {
            Self::Observed(process) => &process.node_id,
            Self::Missing(process) => &process.node_id,
        }
    }

    pub fn state(&self) -> ProcessStateFilter {
        match self {
            Self::Observed(_) => ProcessStateFilter::Observed,
            Self::Missing(_) => ProcessStateFilter::Missing,
        }
    }

    fn cpu_usage_percent(&self) -> f32 {
        match self {
            Self::Observed(process) => process.cpu_usage_percent,
            Self::Missing(_) => 0.0,
        }
    }

    fn memory_bytes(&self) -> u64 {
        match self {
            Self::Observed(process) => process.memory_bytes,
            Self::Missing(_) => 0,
        }
    }
}

pub fn filter_process_rows(
    observed: &[NodeProcessMetrics],
    missing: &[MissingProcessMetric],
    filter: &ProcessFilter,
) -> Vec<ProcessRow> {
    let query = filter.query.trim().to_lowercase();
    let mut rows = observed
        .iter()
        .cloned()
        .map(ProcessRow::Observed)
        .chain(missing.iter().cloned().map(ProcessRow::Missing))
        .filter(|row| filter.state.is_none_or(|state| row.state() == state))
        .filter(|row| !filter.high_cpu || row.cpu_usage_percent() >= 50.0)
        .filter(|row| !filter.high_memory || row.memory_bytes() >= 512 * 1024 * 1024)
        .filter(|row| query.is_empty() || row_matches(row, &query))
        .collect::<Vec<_>>();
    rows.sort_by(process_row_order);
    rows
}

fn process_row_order(left: &ProcessRow, right: &ProcessRow) -> std::cmp::Ordering {
    state_rank(left)
        .cmp(&state_rank(right))
        .then_with(|| {
            right
                .cpu_usage_percent()
                .total_cmp(&left.cpu_usage_percent())
        })
        .then_with(|| right.memory_bytes().cmp(&left.memory_bytes()))
        .then_with(|| row_name(left).cmp(row_name(right)))
}

fn state_rank(row: &ProcessRow) -> u8 {
    match row {
        ProcessRow::Missing(_) => 0,
        ProcessRow::Observed(_) => 1,
    }
}

fn row_matches(row: &ProcessRow, query: &str) -> bool {
    match row {
        ProcessRow::Observed(process) => {
            text_matches(&process.node_id, query)
                || text_matches(&process.node_name, query)
                || text_matches(&process.pid.to_string(), query)
                || text_matches(&format!("{:.1}", process.cpu_usage_percent), query)
                || text_matches(&process.memory_bytes.to_string(), query)
                || text_matches(&process.virtual_memory_bytes.to_string(), query)
                || text_matches(&process.run_time_seconds.to_string(), query)
                || text_matches(&process.status, query)
                || text_matches("observed", query)
        }
        ProcessRow::Missing(process) => {
            text_matches(&process.node_id, query)
                || text_matches(&process.node_name, query)
                || text_matches(&process.pid.to_string(), query)
                || text_matches("missing", query)
        }
    }
}

fn row_name(row: &ProcessRow) -> &str {
    match row {
        ProcessRow::Observed(process) => &process.node_name,
        ProcessRow::Missing(process) => &process.node_name,
    }
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
#[path = "../../tests/unit/metrics/filter/tests.rs"]
mod tests;
