//! [D] ours, `lotr2` D6: cheats for a single-player game. The original has none.
//!
//! Crowns, wood, stone, iron and weapons are realm stockpiles (`Realm` `+0x118` to `+0x140`).
//! Grain, cattle and happiness belong to each county, so those cheats touch every owned county.

use crate::happiness::HAPPINESS_MAX;
use crate::kingdom::Kingdom;
use crate::overview::owned_counties;

pub const CROWNS: i32 = 1_000;
pub const WOOD: i32 = 250;
pub const STONE: i32 = 250;
pub const IRON: i32 = 250;
/// Per weapon type.
pub const WEAPONS: i32 = 20;
/// Per owned county.
pub const GRAIN: i32 = 500;
/// Per owned county.
pub const CATTLE: i32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cheat {
    Crowns,
    Wood,
    Stone,
    Iron,
    Weapons,
    Grain,
    Cattle,
    Happiness,
    Everything,
}

impl Cheat {
    pub const LIST: [Cheat; 9] = [
        Cheat::Crowns,
        Cheat::Wood,
        Cheat::Stone,
        Cheat::Iron,
        Cheat::Weapons,
        Cheat::Grain,
        Cheat::Cattle,
        Cheat::Happiness,
        Cheat::Everything,
    ];

    /// The letter after Cmd. Cmd+Q and Cmd+H stay with macOS, and C opens the menu.
    pub fn key(self) -> char {
        match self {
            Cheat::Crowns => 'G',
            Cheat::Wood => 'W',
            Cheat::Stone => 'S',
            Cheat::Iron => 'I',
            Cheat::Weapons => 'A',
            Cheat::Grain => 'F',
            Cheat::Cattle => 'B',
            Cheat::Happiness => 'J',
            Cheat::Everything => 'R',
        }
    }

    pub fn from_key(c: char) -> Option<Cheat> {
        Cheat::LIST.into_iter().find(|ch| ch.key().eq_ignore_ascii_case(&c))
    }

    pub fn label(self) -> String {
        match self {
            Cheat::Crowns => format!("{CROWNS} crowns"),
            Cheat::Wood => format!("{WOOD} wood"),
            Cheat::Stone => format!("{STONE} stone"),
            Cheat::Iron => format!("{IRON} iron"),
            Cheat::Weapons => format!("{WEAPONS} of each weapon"),
            Cheat::Grain => format!("{GRAIN} grain in each county"),
            Cheat::Cattle => format!("{CATTLE} cattle in each county"),
            Cheat::Happiness => format!("Happiness {HAPPINESS_MAX} in each county"),
            Cheat::Everything => "All of the above".into(),
        }
    }
}

/// Applies `cheat` to `realm`. Returns false and changes nothing for realm 0 or a realm out of range.
pub fn apply(k: &mut Kingdom, realm: u8, cheat: Cheat) -> bool {
    if realm == 0 || realm as usize >= k.realms.len() {
        return false;
    }
    if cheat == Cheat::Everything {
        for c in Cheat::LIST.into_iter().filter(|&c| c != Cheat::Everything) {
            apply(k, realm, c);
        }
        return true;
    }
    let r = &mut k.realms[realm as usize];
    match cheat {
        Cheat::Crowns => r.gold = r.gold.saturating_add(CROWNS),
        Cheat::Wood => r.wood = r.wood.saturating_add(WOOD),
        Cheat::Stone => r.stone = r.stone.saturating_add(STONE),
        Cheat::Iron => r.iron = r.iron.saturating_add(IRON),
        Cheat::Weapons => r.weapons.iter_mut().for_each(|w| *w = w.saturating_add(WEAPONS)),
        Cheat::Grain | Cheat::Cattle | Cheat::Happiness => {
            for id in owned_counties(k, realm) {
                let c = &mut k.counties[id as usize];
                match cheat {
                    Cheat::Grain => c.grain = c.grain.saturating_add(GRAIN),
                    Cheat::Cattle => c.herd = c.herd.saturating_add(CATTLE),
                    _ => c.happiness = HAPPINESS_MAX,
                }
            }
        }
        Cheat::Everything => unreachable!(),
    }
    true
}

#[cfg(test)]
mod tests;
