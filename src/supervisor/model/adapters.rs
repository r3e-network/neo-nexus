//! The adapter registry: which metrics and log-parsing implementations apply to
//! which node type. Plugins are not in here — see `super::process` for why.

use std::collections::HashMap;

use crate::types::NodeType;

use super::{
    log_parsers::{
        LogParserAdapter, NeoCliLogParser, NeoGoLogParser, NeoRsLogParser, NeoXGethLogParser,
        NeoXRethLogParser,
    },
    metrics::{
        MetricsExporterAdapter, NeoCliMetricsAdapter, NeoGoMetricsAdapter, NeoRsMetricsAdapter,
        NeoXGethMetricsAdapter, NeoXRethMetricsAdapter,
    },
    process::{LifecycleAdapter, NoOpLifecycleAdapter},
};

#[derive(Debug, Clone)]
pub struct NodeAdapters {
    /// Generic lifecycle operations (applies to all node types)
    pub lifecycle: std::sync::Arc<dyn LifecycleAdapter>,

    /// Type-specific metrics exporters
    pub metrics: HashMap<NodeType, std::sync::Arc<dyn MetricsExporterAdapter>>,

    /// Type-specific log parsers
    pub log_parser: HashMap<NodeType, std::sync::Arc<dyn LogParserAdapter>>,
}

impl NodeAdapters {
    /// Create a new registry with NoOp stub implementations for backward compatibility.
    pub fn new() -> Self {
        Self {
            lifecycle: std::sync::Arc::<NoOpLifecycleAdapter>::default(),
            metrics: HashMap::new(),
            log_parser: HashMap::new(),
        }
    }

    /// Initialize NodeAdapters with concrete implementations for all 5 node types.
    /// This is the recommended way to set up Phase 2 infrastructure.
    pub fn initialized() -> Self {
        Self {
            lifecycle: std::sync::Arc::<NoOpLifecycleAdapter>::default(),
            metrics: [
                (
                    NodeType::NeoCli,
                    std::sync::Arc::new(NeoCliMetricsAdapter)
                        as std::sync::Arc<dyn MetricsExporterAdapter>,
                ),
                (
                    NodeType::NeoGo,
                    std::sync::Arc::new(NeoGoMetricsAdapter)
                        as std::sync::Arc<dyn MetricsExporterAdapter>,
                ),
                (
                    NodeType::NeoRs,
                    std::sync::Arc::new(NeoRsMetricsAdapter)
                        as std::sync::Arc<dyn MetricsExporterAdapter>,
                ),
                (
                    NodeType::NeoXGeth,
                    std::sync::Arc::new(NeoXGethMetricsAdapter)
                        as std::sync::Arc<dyn MetricsExporterAdapter>,
                ),
                (
                    NodeType::NeoXReth,
                    std::sync::Arc::new(NeoXRethMetricsAdapter)
                        as std::sync::Arc<dyn MetricsExporterAdapter>,
                ),
            ]
            .into_iter()
            .collect(),
            log_parser: [
                (
                    NodeType::NeoCli,
                    std::sync::Arc::new(NeoCliLogParser) as std::sync::Arc<dyn LogParserAdapter>,
                ),
                (
                    NodeType::NeoGo,
                    std::sync::Arc::new(NeoGoLogParser) as std::sync::Arc<dyn LogParserAdapter>,
                ),
                (
                    NodeType::NeoRs,
                    std::sync::Arc::new(NeoRsLogParser) as std::sync::Arc<dyn LogParserAdapter>,
                ),
                (
                    NodeType::NeoXGeth,
                    std::sync::Arc::new(NeoXGethLogParser) as std::sync::Arc<dyn LogParserAdapter>,
                ),
                (
                    NodeType::NeoXReth,
                    std::sync::Arc::new(NeoXRethLogParser) as std::sync::Arc<dyn LogParserAdapter>,
                ),
            ]
            .into_iter()
            .collect(),
        }
    }

    /// Get metrics exporter adapter for given node type, or None if not registered.
    pub fn get_metrics_adapter(&self, node_type: &NodeType) -> Option<&dyn MetricsExporterAdapter> {
        self.metrics.get(node_type).map(|a| a.as_ref())
    }

    /// Get log parser adapter for given node type, or None if not registered.
    pub fn get_log_parser(&self, node_type: &NodeType) -> Option<&dyn LogParserAdapter> {
        self.log_parser.get(node_type).map(|p| p.as_ref())
    }

    /// Register a metrics exporter adapter for a specific node type.
    pub fn with_metrics<A: MetricsExporterAdapter + 'static>(
        mut self,
        node_type: NodeType,
        adapter: A,
    ) -> Self {
        self.metrics.insert(node_type, std::sync::Arc::new(adapter));
        self
    }

    /// Register a log parser adapter for a specific node type.
    pub fn with_log_parser<A: LogParserAdapter + 'static>(
        mut self,
        node_type: NodeType,
        adapter: A,
    ) -> Self {
        self.log_parser
            .insert(node_type, std::sync::Arc::new(adapter));
        self
    }

    /// Replace the default lifecycle adapter.
    pub fn with_lifecycle<A: LifecycleAdapter + 'static>(mut self, adapter: A) -> Self {
        self.lifecycle = std::sync::Arc::new(adapter);
        self
    }

    /// Check if a metrics adapter is registered for this node type.
    pub fn has_metrics(&self, node_type: &NodeType) -> bool {
        self.metrics.contains_key(node_type)
    }

    /// Check if a log parser is registered for this node type.
    pub fn has_log_parser(&self, node_type: &NodeType) -> bool {
        self.log_parser.contains_key(node_type)
    }
}

impl Default for NodeAdapters {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/supervisor/adapters.rs"]
mod tests;
