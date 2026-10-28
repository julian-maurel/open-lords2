use super::*;

const REALM: u8 = 1;

/// The realm owns counties 1 and 2. County 3 belongs to realm 2.
fn world() -> Kingdom {
    let mut k = Kingdom::new(1);
    k.county_count = 3;
    for (id, owner) in [(1u8, REALM), (2, REALM), (3, 2)] {
        let c = &mut k.counties[id as usize];
        c.owner = owner;
        c.grain = 100;
        c.herd = 10;
        c.happiness = 40;
    }
    k.realms[REALM as usize].gold = 500;
    k
}

#[test]
fn realm_cheats_add_to_the_realm_stockpiles_only() {
    let mut k = world();
    let other = k.realms[2].clone();
    for c in [Cheat::Crowns, Cheat::Wood, Cheat::Stone, Cheat::Iron, Cheat::Weapons] {
        assert!(apply(&mut k, REALM, c));
    }
    let r = &k.realms[REALM as usize];
    assert_eq!(r.gold, 500 + CROWNS);
    assert_eq!((r.wood, r.stone, r.iron), (WOOD, STONE, IRON));
    assert!(r.weapons.iter().all(|&w| w == WEAPONS));
    assert_eq!(k.realms[2], other);
}

#[test]
fn county_cheats_touch_every_owned_county_and_no_other() {
    let mut k = world();
    for c in [Cheat::Grain, Cheat::Cattle, Cheat::Happiness] {
        apply(&mut k, REALM, c);
    }
    for id in [1, 2] {
        let c = &k.counties[id];
        assert_eq!((c.grain, c.herd, c.happiness), (100 + GRAIN, 10 + CATTLE, HAPPINESS_MAX));
    }
    let c = &k.counties[3];
    assert_eq!((c.grain, c.herd, c.happiness), (100, 10, 40));
}

#[test]
fn everything_is_every_other_cheat_once() {
    let mut a = world();
    let mut b = world();
    apply(&mut a, REALM, Cheat::Everything);
    for c in Cheat::LIST.into_iter().filter(|&c| c != Cheat::Everything) {
        apply(&mut b, REALM, c);
    }
    assert_eq!(a.realms, b.realms);
    assert_eq!(a.counties, b.counties);
}

#[test]
fn realm_zero_and_a_realm_out_of_range_change_nothing() {
    let mut k = world();
    let before = (k.realms.clone(), k.counties.clone());
    assert!(!apply(&mut k, 0, Cheat::Everything));
    assert!(!apply(&mut k, 200, Cheat::Everything));
    assert_eq!((k.realms.clone(), k.counties.clone()), before);
}

#[test]
fn the_keys_are_distinct_and_leave_the_menu_and_macos_keys_alone() {
    let keys: Vec<char> = Cheat::LIST.iter().map(|c| c.key()).collect();
    for (i, k) in keys.iter().enumerate() {
        assert!(!keys[i + 1..].contains(k), "{k} twice");
        assert!(!"CQHM".contains(*k), "{k} is taken");
        assert_eq!(Cheat::from_key(k.to_ascii_lowercase()), Some(Cheat::LIST[i]));
    }
    assert_eq!(Cheat::from_key('c'), None);
}
