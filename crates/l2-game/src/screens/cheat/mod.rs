//! [D] ours, `lotr2` D6: the cheat menu and the Cmd+letter cheats. The original has neither;
//! the rules are [`l2_kingdom::cheat`].
//!
//! Cmd+C opens the menu over the campaign map. Each other letter in [`Cheat::key`] applies its
//! cheat at once, with the menu open or not. Only a single-player game in progress takes them.

use l2_kingdom::cheat::Cheat;
use l2_view::Canvas;

use crate::input::{Event, Key, Rect};
use crate::screen::{Ctx, Screen, ScreenId, Transition};
use crate::shell::{font, Pen};
use crate::Game;

pub const MENU_KEY: char = 'C';

pub const BOX_X: i32 = 8;
pub const BOX_Y: i32 = 28;
pub const BOX_COLS: i32 = 27;
pub const BOX_ROWS: i32 = 19;
pub const LEFT: i32 = BOX_X + 16;
pub const RIGHT: i32 = BOX_X + BOX_COLS * 16 - 14;
pub const BOTTOM: i32 = BOX_Y + BOX_ROWS * 16 - 12;
pub const ROW_Y0: i32 = BOX_Y + 44;
pub const ROW_H: i32 = 18;
pub const LABEL_X: i32 = LEFT + 64;
pub const OK: Rect = Rect::new(RIGHT - 24, BOTTOM - 24, 24, 24);

/// The line of `cheat` in the menu.
pub fn row(cheat: Cheat) -> Rect {
    let i = Cheat::LIST.iter().position(|&c| c == cheat).unwrap_or(0) as i32;
    Rect::new(LEFT, ROW_Y0 + i * ROW_H, RIGHT - LEFT, ROW_H)
}

/// A single-player game on the campaign map, with no battle up.
pub fn allowed(ids: &[ScreenId], game: &Game) -> bool {
    !game.multiplayer
        && !game.net.lockstep
        && game.player != 0
        && ids.contains(&ScreenId::Campaign)
        && !ids.contains(&ScreenId::Battlefield)
}

/// The machine calls this for Cmd+`c` while the menu is closed. True asks it to open the menu.
pub fn shortcut(c: char, ids: &[ScreenId], game: &mut Game) -> bool {
    if !allowed(ids, game) {
        return false;
    }
    if c.eq_ignore_ascii_case(&MENU_KEY) {
        return true;
    }
    if let Some(cheat) = Cheat::from_key(c) {
        cheat_now(game, cheat);
    }
    false
}

fn cheat_now(game: &mut Game, cheat: Cheat) {
    let Some(i) = Cheat::LIST.iter().position(|&c| c == cheat) else { return };
    if game.issue(crate::command::ApplyCheat { cheat: i as u8 }) {
        eprintln!("cheat: {}", cheat.label());
    }
}

#[derive(Default)]
pub struct CheatScreen;

impl CheatScreen {
    pub fn new() -> CheatScreen {
        CheatScreen
    }
}

impl Screen for CheatScreen {
    fn id(&self) -> ScreenId {
        ScreenId::Cheat
    }

    fn title(&self, _ctx: &Ctx) -> String {
        "Cheats".into()
    }

    fn is_overlay(&self) -> bool {
        true
    }

    fn handle(&mut self, event: Event, ctx: &mut Ctx) -> Transition {
        match event {
            Event::RightClick { .. } => Transition::Pop,
            Event::KeyDown(Key::Char(c) | Key::CmdChar(c)) => {
                if c.eq_ignore_ascii_case(&MENU_KEY) {
                    return Transition::Pop;
                }
                if let Some(cheat) = Cheat::from_key(c) {
                    cheat_now(ctx.game, cheat);
                }
                Transition::Stay
            }
            Event::Click { x, y } => {
                if OK.contains(x, y) {
                    return Transition::Pop;
                }
                if let Some(&cheat) = Cheat::LIST.iter().find(|&&c| row(c).contains(x, y)) {
                    cheat_now(ctx.game, cheat);
                }
                Transition::Stay
            }
            _ => Transition::Stay,
        }
    }

    fn draw(&mut self, ctx: &Ctx, canvas: &mut Canvas) {
        let pen = Pen {
            assets: &ctx.assets.shell,
            ink: &ctx.assets.ink,
            chrome: ctx.assets.chrome.as_ref(),
            shadow: Some(font::SHADOW),
            caps: None,
        };
        pen.window(canvas, BOX_X, BOX_Y, BOX_COLS, BOX_ROWS, 0);
        pen.heading(canvas, LEFT, BOX_Y + 12, "Cheats", font::TEXT);
        for cheat in Cheat::LIST {
            let r = row(cheat);
            pen.body(canvas, LEFT, r.y, &format!("Cmd+{}", cheat.key()), font::HIGHLIGHT);
            pen.body(canvas, LABEL_X, r.y, &cheat.label(), font::TEXT);
        }
        let mut y = ROW_Y0 + Cheat::LIST.len() as i32 * ROW_H + 6;
        if let Some(r) = ctx.game.kingdom.realms.get(ctx.game.player as usize) {
            let line = format!("Treasury {} crowns.", r.gold);
            pen.body(canvas, LEFT, y, &line, font::TEXT);
            y += ROW_H;
            let line = format!("Wood {}, stone {}, iron {}.", r.wood, r.stone, r.iron);
            pen.body(canvas, LEFT, y, &line, font::TEXT);
            y += ROW_H;
        }
        pen.body(canvas, LEFT, y, "Click a line or press its letter.", font::TEXT);
        pen.ok_button(canvas, OK.x, OK.y, 0);
    }
}

#[cfg(test)]
mod tests;
