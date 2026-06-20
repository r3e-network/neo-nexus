use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResourcePressure {
    Nominal,
    Elevated,
    Critical,
}

impl ResourcePressure {
    pub fn from_percent(percent: f32) -> Self {
        if percent >= 92.0 {
            Self::Critical
        } else if percent >= 78.0 {
            Self::Elevated
        } else {
            Self::Nominal
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Nominal => "nominal",
            Self::Elevated => "elevated",
            Self::Critical => "critical",
        }
    }
}
