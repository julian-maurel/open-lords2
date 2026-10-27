//! [D] ours, `lotr2` D5: the realm ledger and the end-of-turn alerts. The original has
//! neither; the numbers come from [`l2_kingdom::overview`], which only reads the kingdom.
//!
//! The screen issues no command. A row or an alert selects its county, and the end-turn
//! view's tick asks the map to end the turn through [`crate::turn_clock::TurnClock::defer_end_turn`],
//! the path `Turn_End` (`0x0043AC23`) already takes while a turn opens.

mod draw;

use l2_kingdom::overview::{alerts, overview, Alert, Overview};
use l2_view::Canvas;

use crate::input::{Event, Key, Rect};
use crate::screen::{Ctx, Screen, ScreenId, Transition};
use crate::screens::battlefield::{CONFIRM_NO_FRAME, CONFIRM_YES_FRAME};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealmView {
    /// The table of owned counties, then the alerts.
    Ledger,
    /// The alerts, and a yes/no on ending the turn.
    EndTurn,
}

pub const KEY: char = 'r';

pub const BOX_X: i32 = 8;
pub const BOX_Y: i32 = 28;
/// The table needs about 560 pixels in the body font, so the window covers the right column too.
pub const BOX_COLS: i32 = 39;
pub const BOX_ROWS: i32 = 28;
pub const ROW_Y0: i32 = 92;
pub const ROW_H: i32 = 17;
pub const ALERT_H: i32 = 16;
pub const LEFT: i32 = BOX_X + 16;
pub const RIGHT: i32 = BOX_X + BOX_COLS * 16 - 14;
pub const BOTTOM: i32 = BOX_Y + BOX_ROWS * 16 - 12;
pub const OK: Rect = Rect::new(RIGHT - 24, BOTTOM - 24, 24, 24);
/// The end-turn view is smaller: it leaves the right column in sight.
pub const END_COLS: i32 = 29;
pub const END_ROWS: i32 = 14;
pub const END_RIGHT: i32 = BOX_X + END_COLS * 16 - 14;
pub const END_BOTTOM: i32 = BOX_Y + END_ROWS * 16 - 12;
pub const YES: Rect = Rect::new(END_RIGHT - 84, END_BOTTOM - 34, 32, 32);
pub const NO: Rect = Rect::new(END_RIGHT - 36, END_BOTTOM - 30, 32, 32);
pub const YES_FRAME: usize = CONFIRM_YES_FRAME;
pub const NO_FRAME: usize = CONFIRM_NO_FRAME;

/// The map's End Turn arm asks this first: no alert, no question.
pub fn has_alerts(ctx: &Ctx) -> bool {
    !alerts(&overview(&ctx.game.kingdom, ctx.game.player)).is_empty()
}

pub struct RealmScreen {
    view: RealmView,
    /// Each line that selects a county, with the rectangle it was drawn in.
    hits: Vec<(Rect, u8)>,
}

impl RealmScreen {
    pub fn new(view: RealmView) -> RealmScreen {
        RealmScreen { view, hits: Vec::new() }
    }

    fn data(ctx: &Ctx) -> (Overview, Vec<Alert>) {
        let o = overview(&ctx.game.kingdom, ctx.game.player);
        let a = alerts(&o);
        (o, a)
    }

    fn select(ctx: &mut Ctx, county: u8) -> Transition {
        ctx.game.selected = county;
        Transition::Pop
    }
}

impl Screen for RealmScreen {
    fn id(&self) -> ScreenId {
        ScreenId::Realm(self.view)
    }

    fn title(&self, _ctx: &Ctx) -> String {
        "Realm ledger".into()
    }

    fn is_overlay(&self) -> bool {
        true
    }

    fn handle(&mut self, event: Event, ctx: &mut Ctx) -> Transition {
        match event {
            Event::RightClick { .. } => Transition::Pop,
            Event::KeyDown(Key::Char(c)) if c.eq_ignore_ascii_case(&KEY) => Transition::Pop,
            Event::KeyDown(Key::Enter) if self.view == RealmView::EndTurn => {
                ctx.game.turn_clock.defer_end_turn();
                Transition::Pop
            }
            Event::Click { x, y } => {
                if self.view == RealmView::EndTurn {
                    if YES.contains(x, y) {
                        ctx.game.turn_clock.defer_end_turn();
                        return Transition::Pop;
                    }
                    if NO.contains(x, y) {
                        return Transition::Pop;
                    }
                } else if OK.contains(x, y) {
                    return Transition::Pop;
                }
                match self.hits.iter().find(|(r, _)| r.contains(x, y)) {
                    Some(&(_, county)) => Self::select(ctx, county),
                    None => Transition::Stay,
                }
            }
            _ => Transition::Stay,
        }
    }

    fn draw(&mut self, ctx: &Ctx, canvas: &mut Canvas) {
        let (o, a) = Self::data(ctx);
        self.hits.clear();
        draw::draw(self, ctx, canvas, &o, &a);
    }
}

#[cfg(test)]
mod tests;
