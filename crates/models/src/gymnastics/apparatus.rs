use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Apparatus {
    Vault,
    UnevenBars,
    BalanceBeam,
    Floor,
    PommelHorse,
    StillRings,
    ParallelBars,
    HorizontalBar,
}

impl Apparatus {
    pub fn label(&self) -> &'static str {
        match self {
            Apparatus::Vault => "Vault",
            Apparatus::UnevenBars => "Uneven Bars",
            Apparatus::BalanceBeam => "Balance Beam",
            Apparatus::Floor => "Floor Exercise",
            Apparatus::PommelHorse => "Pommel Horse",
            Apparatus::StillRings => "Still Rings",
            Apparatus::ParallelBars => "Parallel Bars",
            Apparatus::HorizontalBar => "Horizontal Bar",
        }
    }
}
