//! The integration suite drives the library through its public API only: a
//! real SQLite workspace, the real launch planner, config exporter and process
//! supervisor, and a real child process standing in for a node runtime.

mod common;
mod lifecycle;
mod node_types;
