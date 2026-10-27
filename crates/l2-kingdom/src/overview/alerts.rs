//! [D] ours, `lotr2` D5: what the player may want to fix before ending the turn.

use super::{CountyRow, Overview};
use crate::unrest::{HUMAN_UNREST_BELOW, HUMAN_WARN_BELOW};
pub use crate::unrest::UNREST_REVOLT;

/// The grain alert fires when the store covers this many seasons or fewer.
pub const GRAIN_SEASONS_LOW: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Danger,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alert {
    /// Happiness under 25 steps the unrest counter; at [`UNREST_REVOLT`] the county revolts
    /// (`unrest::update`). `unrest` is the counter now, before the step.
    Revolt { county: u8, happiness: i32, unrest: u8 },
    /// Happiness under 30 and at or over 25: a warning, with no step.
    Unhappy { county: u8, happiness: i32 },
    /// The ration forecast for next season is under the level the player set.
    Hungry { county: u8, wanted: i32, achieved: i32 },
    GrainLow { county: u8, seasons: i32 },
    /// The tax panel shows a figure that the tax pass will not collect.
    TaxSuppressed { county: u8, shown: i32 },
    /// The tax pass runs before the wage pass, so its take counts towards the bill.
    Wages { gold: i32, income: i32, wages: i32 },
    Enemy { county: u8, men: i32, bridge: bool },
}

impl Alert {
    pub fn severity(&self) -> Severity {
        match *self {
            Alert::Revolt { .. } | Alert::Wages { .. } | Alert::Enemy { .. } => Severity::Danger,
            Alert::Hungry { achieved, .. } if achieved == 0 => Severity::Danger,
            _ => Severity::Warning,
        }
    }

    pub fn county(&self) -> Option<u8> {
        match *self {
            Alert::Revolt { county, .. }
            | Alert::Unhappy { county, .. }
            | Alert::Hungry { county, .. }
            | Alert::GrainLow { county, .. }
            | Alert::TaxSuppressed { county, .. }
            | Alert::Enemy { county, .. } => Some(county),
            Alert::Wages { .. } => None,
        }
    }
}

pub fn alerts(o: &Overview) -> Vec<Alert> {
    let mut out = Vec::new();
    let t = &o.totals;
    if t.wages > 0 && t.gold + t.tax_expected < t.wages {
        out.push(Alert::Wages { gold: t.gold, income: t.tax_expected, wages: t.wages });
    }
    for r in &o.rows {
        county_alerts(r, &mut out);
    }
    out.sort_by_key(|a| (a.severity(), a.county()));
    out
}

fn county_alerts(r: &CountyRow, out: &mut Vec<Alert>) {
    let county = r.county;
    if r.enemy_troops > 0 {
        out.push(Alert::Enemy { county, men: r.enemy_troops, bridge: r.bridge });
    }
    if r.happiness < HUMAN_UNREST_BELOW {
        out.push(Alert::Revolt { county, happiness: r.happiness, unrest: r.unrest });
    } else if r.happiness < HUMAN_WARN_BELOW {
        out.push(Alert::Unhappy { county, happiness: r.happiness });
    }
    if r.ration_achieved < r.ration_wanted {
        out.push(Alert::Hungry { county, wanted: r.ration_wanted, achieved: r.ration_achieved });
    }
    if let Some(seasons) = r.grain_seasons().filter(|&s| s <= GRAIN_SEASONS_LOW) {
        out.push(Alert::GrainLow { county, seasons });
    }
    if r.tax_suppressed && r.tax_shown > 0 {
        out.push(Alert::TaxSuppressed { county, shown: r.tax_shown });
    }
}
