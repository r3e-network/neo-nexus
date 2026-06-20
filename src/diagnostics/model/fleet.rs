use super::NodeDiagnostics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FleetDiagnostics {
    pub score: usize,
    pub ready_nodes: usize,
    pub warning_count: usize,
    pub critical_count: usize,
    pub nodes: Vec<NodeDiagnostics>,
}
