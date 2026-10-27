//! [D] ours, `lotr2` D5: a read-only summary of one realm, for the realm ledger and the
//! end-of-turn alerts. The original has no such screen. Nothing here writes to the kingdom.

mod alerts;
pub use alerts::*;

use crate::county::County;
use crate::kingdom::Kingdom;
use crate::territory::is_neighbour;

/// One owned county, as the ledger shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountyRow {
    pub county: u8,
    pub population: i32,
    pub population_change: i32,
    pub happiness: i32,
    pub happiness_change: i32,
    pub health_band: u8,
    pub ration_wanted: i32,
    /// Next season's forecast: `ration::preview` writes it during the player's turn (C20).
    pub ration_achieved: i32,
    pub grain: i32,
    pub herd: i32,
    pub grain_eaten: i32,
    pub herd_eaten: i32,
    pub tax_rate: i32,
    /// "People pay": the tax panel's figure, `Tax_RecomputePreview` (`0x0044B80B`).
    pub tax_shown: i32,
    /// What the tax pass will take: the panel's figure, or 0 when `tax_suppressed` stops it.
    pub tax_expected: i32,
    pub tax_suppressed: bool,
    pub unrest: u8,
    pub friendly_troops: i32,
    pub enemy_troops: i32,
    /// Losing this county cuts the realm in two, and the smaller part secedes (§6.1).
    pub bridge: bool,
}

impl CountyRow {
    /// Whole seasons the grain store lasts at next season's consumption, or `None`
    /// when the county eats no grain.
    pub fn grain_seasons(&self) -> Option<i32> {
        (self.grain_eaten > 0).then(|| self.grain.max(0) / self.grain_eaten)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Totals {
    pub population: i32,
    pub population_change: i32,
    pub grain: i32,
    pub herd: i32,
    pub tax_expected: i32,
    pub gold: i32,
    /// The army bill the wage pass will charge, from `wages_for_realm`.
    pub wages: i32,
    pub men: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Overview {
    pub realm: u8,
    pub rows: Vec<CountyRow>,
    pub totals: Totals,
}

pub fn owned_counties(k: &Kingdom, realm: u8) -> Vec<u8> {
    (1..=k.county_count.min(k.counties.len() - 1))
        .filter(|&c| realm != 0 && k.counties[c].owner == realm)
        .map(|c| c as u8)
        .collect()
}

pub fn overview(k: &Kingdom, realm: u8) -> Overview {
    let owned = owned_counties(k, realm);
    let rows: Vec<CountyRow> =
        owned.iter().map(|&c| row(&k.counties[c as usize], c, is_bridge(&k.counties, &owned, c))).collect();
    let mut totals = Totals::default();
    for r in &rows {
        totals.population += r.population;
        totals.population_change += r.population_change;
        totals.grain += r.grain;
        totals.herd += r.herd;
        totals.tax_expected += r.tax_expected;
    }
    if let Some(r) = k.realms.get(realm as usize) {
        totals.gold = r.gold;
        totals.wages = crate::unit::wages_for_realm(
            &k.tables,
            &k.campaign.units,
            &k.realms,
            realm,
            k.options.difficulty,
        );
        totals.men = k.campaign.units.realm_totals(realm).1;
    }
    Overview { realm, rows, totals }
}

fn row(c: &County, id: u8, bridge: bool) -> CountyRow {
    CountyRow {
        county: id,
        population: c.population,
        population_change: c.population - c.pop_last,
        happiness: c.happiness,
        happiness_change: c.happiness - c.happiness_last,
        health_band: c.health_band,
        ration_wanted: c.ration_wanted,
        ration_achieved: c.ration_achieved,
        grain: c.grain,
        herd: c.herd,
        grain_eaten: c.grain_eaten,
        herd_eaten: c.herd_eaten,
        tax_rate: c.tax_rate,
        tax_shown: c.tax_shown,
        tax_expected: if c.tax_suppressed { 0 } else { c.tax_shown },
        tax_suppressed: c.tax_suppressed,
        unrest: c.unrest,
        friendly_troops: c.friendly_troops,
        enemy_troops: c.enemy_troops,
        bridge,
    }
}

/// True when the owned counties other than `cut` form more than one connected block.
pub fn is_bridge(counties: &[County], owned: &[u8], cut: u8) -> bool {
    let rest: Vec<u8> = owned.iter().copied().filter(|&c| c != cut).collect();
    let Some(&first) = rest.first() else { return false };
    let mut reached = vec![first];
    let mut i = 0;
    while i < reached.len() {
        let from = reached[i];
        for &to in &rest {
            let joined = is_neighbour(counties, from, to) || is_neighbour(counties, to, from);
            if joined && !reached.contains(&to) {
                reached.push(to);
            }
        }
        i += 1;
    }
    reached.len() < rest.len()
}

#[cfg(test)]
mod tests;
