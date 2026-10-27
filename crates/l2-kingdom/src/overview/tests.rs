use super::*;

const REALM: u8 = 1;

/// Five counties in a line, 1-2-3-4-5. The realm owns 1, 2 and 3; 4 and 5 are unowned.
fn line() -> Kingdom {
    let mut k = Kingdom::new(1);
    k.county_count = 5;
    for id in 1..=5u8 {
        let c = &mut k.counties[id as usize];
        c.owner = if id <= 3 { REALM } else { 0 };
        c.population = 400;
        c.pop_last = 400;
        c.happiness = 60;
        c.happiness_last = 60;
        c.ration_wanted = 3;
        c.ration_achieved = 3;
        c.grain = 100;
        c.grain_eaten = 10;
        if id > 1 {
            c.add_neighbour(id - 1);
        }
        if id < 5 {
            c.add_neighbour(id + 1);
        }
    }
    k.realms[REALM as usize].gold = 500;
    k
}

#[test]
fn the_overview_lists_the_owned_counties_only() {
    let o = overview(&line(), REALM);
    assert_eq!(o.rows.iter().map(|r| r.county).collect::<Vec<_>>(), vec![1, 2, 3]);
    assert_eq!(o.totals.population, 1200);
    assert_eq!(o.totals.gold, 500);
}

#[test]
fn the_middle_of_a_line_is_a_bridge_and_its_ends_are_not() {
    let o = overview(&line(), REALM);
    let bridges: Vec<bool> = o.rows.iter().map(|r| r.bridge).collect();
    assert_eq!(bridges, vec![false, true, false]);
}

#[test]
fn a_ring_has_no_bridge() {
    let mut k = line();
    k.counties[1].add_neighbour(3);
    k.counties[3].add_neighbour(1);
    assert!(overview(&k, REALM).rows.iter().all(|r| !r.bridge));
}

#[test]
fn a_quiet_realm_raises_no_alert() {
    assert_eq!(alerts(&overview(&line(), REALM)), vec![]);
}

#[test]
fn happiness_under_25_is_a_revolt_and_under_30_a_warning() {
    let mut k = line();
    k.counties[1].happiness = 24;
    k.counties[1].unrest = 2;
    k.counties[2].happiness = 29;
    k.counties[3].happiness = 30;
    let a = alerts(&overview(&k, REALM));
    assert_eq!(
        a,
        vec![
            Alert::Revolt { county: 1, happiness: 24, unrest: 2 },
            Alert::Unhappy { county: 2, happiness: 29 },
        ]
    );
}

#[test]
fn a_ration_forecast_under_the_wanted_level_is_hunger() {
    let mut k = line();
    k.counties[2].ration_achieved = 1;
    assert_eq!(
        alerts(&overview(&k, REALM)),
        vec![Alert::Hungry { county: 2, wanted: 3, achieved: 1 }]
    );
    k.counties[2].ration_achieved = 0;
    assert_eq!(alerts(&overview(&k, REALM))[0].severity(), Severity::Danger);
}

#[test]
fn grain_for_one_season_or_less_is_low() {
    let mut k = line();
    k.counties[3].grain = 19;
    assert_eq!(alerts(&overview(&k, REALM)), vec![Alert::GrainLow { county: 3, seasons: 1 }]);
    k.counties[3].grain = 20;
    assert_eq!(alerts(&overview(&k, REALM)), vec![]);
    k.counties[3].grain_eaten = 0;
    k.counties[3].grain = 0;
    assert_eq!(alerts(&overview(&k, REALM)), vec![], "a county that eats no grain");
}

#[test]
fn a_suppressed_tax_is_shown_and_not_counted() {
    let mut k = line();
    k.counties[1].tax_shown = 40;
    k.counties[2].tax_shown = 30;
    k.counties[2].tax_suppressed = true;
    let o = overview(&k, REALM);
    assert_eq!(o.totals.tax_expected, 40);
    assert_eq!(alerts(&o), vec![Alert::TaxSuppressed { county: 2, shown: 30 }]);
}

#[test]
fn wages_count_this_seasons_tax_before_the_bill() {
    let mut o = overview(&line(), REALM);
    o.totals.gold = 50;
    o.totals.tax_expected = 49;
    o.totals.wages = 100;
    assert_eq!(alerts(&o), vec![Alert::Wages { gold: 50, income: 49, wages: 100 }]);
    o.totals.tax_expected = 50;
    assert_eq!(alerts(&o), vec![]);
}

#[test]
fn danger_comes_before_warning() {
    let mut k = line();
    k.counties[1].happiness = 28;
    k.counties[3].enemy_troops = 120;
    k.counties[2].enemy_troops = 5;
    let a = alerts(&overview(&k, REALM));
    assert_eq!(
        a,
        vec![
            Alert::Enemy { county: 2, men: 5, bridge: true },
            Alert::Enemy { county: 3, men: 120, bridge: false },
            Alert::Unhappy { county: 1, happiness: 28 },
        ]
    );
}
