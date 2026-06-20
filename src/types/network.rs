use std::{fmt, str::FromStr};

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
    Private,
}

impl Network {
    pub const ALL: [Self; 3] = [Self::Mainnet, Self::Testnet, Self::Private];
}

impl fmt::Display for Network {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
            Self::Private => "private",
        })
    }
}

impl FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            "private" => Ok(Self::Private),
            other => anyhow::bail!("unsupported network: {other}"),
        }
    }
}
