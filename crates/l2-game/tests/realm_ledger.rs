//! [D] ours, `lotr2` D5: the realm ledger and the alerts that stand in front of End Turn.

use l2_game::game::Assets;
use l2_game::input::{Event, Key};
use l2_game::screen::{Ctx, Machine, ScreenId};
use l2_game::screens::map::END_TURN_BUTTON;
use l2_game::screens::realm::{RealmView, NO, YES};
use l2_game::Game;
use l2_kingdom::AI_STEP_DONE;

fn world(alerts: bool) -> (Game, Assets) {
    let mut g = Game::new(5);
    g.prefs.tip_screens = false;
    g.prefs.end_turn_alerts = alerts;
    g.kingdom.set_county_count(3);
    l2_testkit::chain_neighbours!(g.kingdom);
    for r in [1, 2] {
        g.kingdom.realms[r].in_play = true;
        g.kingdom.realms[r].strength = 3;
    }
    g.kingdom.realms[1].is_human = true;
    for (c, owner) in [(1, 1), (2, 0), (3, 2)] {
        g.kingdom.counties[c].owner = owner;
        g.kingdom.counties[c].population = 400;
    }
    // Under 25: a revolt alert.
    g.kingdom.counties[1].happiness = 10;
    g.player = 1;
    g.selected = 1;
    (g, Assets::placeholder())
}

fn ticks(m: &mut Machine, game: &mut Game, assets: &Assets, n: usize) {
    for _ in 0..n {
        m.update(&mut Ctx { game, assets });
    }
}

fn send(m: &mut Machine, game: &mut Game, assets: &Assets, e: Event) {
    m.handle(e, &mut Ctx { game, assets });
}

#[test]
fn end_turn_with_an_alert_asks_first_and_yes_ends_the_turn() {
    let (mut game, assets) = world(true);
    let mut m = Machine::new(ScreenId::Campaign);
    ticks(&mut m, &mut game, &assets, 40);

    send(&mut m, &mut game, &assets, END_TURN_BUTTON.click());
    assert_eq!(m.top_id(), Some(ScreenId::Realm(RealmView::EndTurn)));
    ticks(&mut m, &mut game, &assets, 1);
    assert_eq!(game.kingdom.realms[1].ai_step, 1, "the turn has not ended under the question");

    send(&mut m, &mut game, &assets, YES.click());
    ticks(&mut m, &mut game, &assets, 1);
    assert_eq!(m.top_id(), Some(ScreenId::Campaign));
    assert_eq!(game.kingdom.realms[1].ai_step, AI_STEP_DONE, "Turn_End (0x0043AC23) ran");
}

#[test]
fn no_keeps_the_turn_open() {
    let (mut game, assets) = world(true);
    let mut m = Machine::new(ScreenId::Campaign);
    ticks(&mut m, &mut game, &assets, 40);
    send(&mut m, &mut game, &assets, END_TURN_BUTTON.click());
    send(&mut m, &mut game, &assets, NO.click());
    ticks(&mut m, &mut game, &assets, 2);
    assert_eq!(m.top_id(), Some(ScreenId::Campaign));
    assert_eq!(game.kingdom.realms[1].ai_step, 1);
}

#[test]
fn with_the_preference_off_end_turn_is_the_original() {
    let (mut game, assets) = world(false);
    let mut m = Machine::new(ScreenId::Campaign);
    ticks(&mut m, &mut game, &assets, 40);
    send(&mut m, &mut game, &assets, END_TURN_BUTTON.click());
    ticks(&mut m, &mut game, &assets, 1);
    assert_eq!(game.kingdom.realms[1].ai_step, AI_STEP_DONE);
}

#[test]
fn r_opens_and_closes_the_ledger() {
    let (mut game, assets) = world(true);
    let mut m = Machine::new(ScreenId::Campaign);
    ticks(&mut m, &mut game, &assets, 40);
    send(&mut m, &mut game, &assets, Event::KeyDown(Key::Char('r')));
    assert_eq!(m.top_id(), Some(ScreenId::Realm(RealmView::Ledger)));
    send(&mut m, &mut game, &assets, Event::KeyDown(Key::Char('R')));
    assert_eq!(m.top_id(), Some(ScreenId::Campaign));
}

/// Writes `realm_ledger.ppm` and `realm_end_turn.ppm` into `LORDS2_SHOTS`, drawn with the
/// install's own art. `cargo test -p l2-game --test realm_ledger -- --ignored`.
#[test]
#[ignore]
fn shoot() {
    let Some(dir) = l2_testkit::install_dir() else { l2_testkit::skip!("no game install") };
    let out = std::env::var_os("LORDS2_SHOTS").map_or_else(|| "out".into(), std::path::PathBuf::from);
    let platform = l2_mods::Platform::builder().base(&dir).build().expect("the install mounts");
    let assets = Assets::load(&platform.vfs).expect("assets load");
    let settings = l2_game::setup::SetupOptions::new().commit(1, l2_kingdom::Quirks::default());
    let tables = platform.kingdom_tables().expect("tables");
    let mut game = l2_game::scenario::new_game(&assets, 0, &settings, 1, 1, 7, tables).expect("a game");
    game.prefs.tip_screens = false;
    let home = (1..=game.kingdom.county_count as u8).find(|&c| game.is_players(c)).expect("a county");
    let mut owned = vec![home];
    owned.extend(game.kingdom.counties[home as usize].neighbours().iter().copied().take(4));
    for &c in &owned {
        game.kingdom.counties[c as usize].owner = game.player;
    }
    let owned = l2_kingdom::overview::owned_counties(&game.kingdom, game.player);
    let k = &mut game.kingdom;
    for &c in &owned {
        let c = &mut k.counties[c as usize];
        (c.happiness, c.ration_wanted, c.ration_achieved, c.grain, c.grain_eaten) = (60, 3, 3, 400, 40);
        c.tax_rate = 5;
        c.tax_shown = 25;
    }
    let pick = |i: usize| owned[i.min(owned.len() - 1)] as usize;
    k.counties[pick(1)].happiness = 22;
    k.counties[pick(1)].unrest = 2;
    k.counties[pick(2)].ration_achieved = 1;
    k.counties[pick(3)].grain = 50;
    k.counties[pick(3)].enemy_troops = 150;
    k.counties[pick(0)].tax_suppressed = true;
    k.counties[pick(0)].friendly_troops = 300;
    std::fs::create_dir_all(&out).unwrap();
    for (name, view) in [("realm_ledger", RealmView::Ledger), ("realm_end_turn", RealmView::EndTurn)] {
        let mut m = Machine::new(ScreenId::Campaign);
        m.push(ScreenId::Realm(view));
        let mut canvas = l2_view::Canvas::screen();
        m.draw(&Ctx { game: &mut game, assets: &assets }, &mut canvas);
        let mut rgba = vec![0u8; 640 * 480 * 4];
        canvas.to_rgba(&assets.palette, &mut rgba);
        let mut ppm = b"P6 640 480 255\n".to_vec();
        ppm.extend(rgba.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]));
        std::fs::write(out.join(format!("{name}.ppm")), ppm).unwrap();
    }
}
