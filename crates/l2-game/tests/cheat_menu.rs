//! [D] ours, `lotr2` D6: the cheat menu and the Cmd+letter cheats.

use l2_game::game::Assets;
use l2_game::input::{Event, Key};
use l2_game::screen::{Ctx, Machine, ScreenId};
use l2_game::screens::cheat::{row, OK};
use l2_game::Game;
use l2_kingdom::cheat::{Cheat, CROWNS, GRAIN, WOOD};

fn world() -> (Game, Assets) {
    let mut g = Game::new(5);
    g.prefs.tip_screens = false;
    g.kingdom.set_county_count(3);
    l2_testkit::chain_neighbours!(g.kingdom);
    for r in [1, 2] {
        g.kingdom.realms[r].in_play = true;
        g.kingdom.realms[r].strength = 3;
    }
    g.kingdom.realms[1].is_human = true;
    g.kingdom.realms[1].gold = 500;
    for (c, owner) in [(1, 1), (2, 1), (3, 2)] {
        g.kingdom.counties[c].owner = owner;
        g.kingdom.counties[c].population = 400;
        g.kingdom.counties[c].happiness = 60;
    }
    g.player = 1;
    g.selected = 1;
    (g, Assets::placeholder())
}

fn campaign() -> (Machine, Game, Assets) {
    let (mut game, assets) = world();
    let mut m = Machine::new(ScreenId::Campaign);
    for _ in 0..40 {
        m.update(&mut Ctx { game: &mut game, assets: &assets });
    }
    (m, game, assets)
}

fn send(m: &mut Machine, game: &mut Game, assets: &Assets, e: Event) {
    m.handle(e, &mut Ctx { game, assets });
}

fn cmd(c: char) -> Event {
    Event::KeyDown(Key::cmd_letter(c))
}

#[test]
fn cmd_c_opens_the_menu_and_closes_it_again() {
    let (mut m, mut game, assets) = campaign();
    send(&mut m, &mut game, &assets, cmd('c'));
    assert_eq!(m.top_id(), Some(ScreenId::Cheat));
    send(&mut m, &mut game, &assets, cmd('c'));
    assert_eq!(m.top_id(), Some(ScreenId::Campaign));
    send(&mut m, &mut game, &assets, cmd('c'));
    send(&mut m, &mut game, &assets, OK.click());
    assert_eq!(m.top_id(), Some(ScreenId::Campaign));
}

#[test]
fn a_shortcut_works_without_the_menu_and_the_menu_takes_clicks_and_letters() {
    let (mut m, mut game, assets) = campaign();
    send(&mut m, &mut game, &assets, cmd('g'));
    assert_eq!(m.top_id(), Some(ScreenId::Campaign), "no menu for a shortcut");
    assert_eq!(game.kingdom.realms[1].gold, 500 + CROWNS);

    send(&mut m, &mut game, &assets, cmd('c'));
    send(&mut m, &mut game, &assets, row(Cheat::Wood).click());
    send(&mut m, &mut game, &assets, Event::KeyDown(Key::letter('w')));
    send(&mut m, &mut game, &assets, cmd('f'));
    assert_eq!(game.kingdom.realms[1].wood, 2 * WOOD);
    assert_eq!(game.kingdom.counties[1].grain, GRAIN);
    assert_eq!(game.kingdom.counties[2].grain, GRAIN);
    assert_eq!(game.kingdom.counties[3].grain, 0, "not the player's county");
    assert_eq!(m.top_id(), Some(ScreenId::Cheat));
}

#[test]
fn a_network_game_takes_no_cheat() {
    let (mut m, mut game, assets) = campaign();
    game.multiplayer = true;
    send(&mut m, &mut game, &assets, cmd('c'));
    send(&mut m, &mut game, &assets, cmd('g'));
    assert_eq!(m.top_id(), Some(ScreenId::Campaign));
    assert_eq!(game.kingdom.realms[1].gold, 500);
}
