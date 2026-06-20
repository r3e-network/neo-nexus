use super::*;

pub(in crate::app) fn view_for_resolution(resolution: DiagnosticResolution) -> View {
    match resolution {
        DiagnosticResolution::ConfigWorkspace => View::Config,
        DiagnosticResolution::Logs => View::Logs,
        DiagnosticResolution::Monitor => View::Monitor,
        DiagnosticResolution::NodeStudio => View::Nodes,
        DiagnosticResolution::Operations => View::Operations,
        DiagnosticResolution::PluginManager => View::Plugins,
        DiagnosticResolution::RolePlanner => View::Roles,
        DiagnosticResolution::RuntimeManager => View::Runtimes,
        DiagnosticResolution::WalletProfiles => View::Wallets,
    }
}
