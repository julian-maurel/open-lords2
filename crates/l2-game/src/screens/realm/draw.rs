use l2_kingdom::overview::{Alert, CountyRow, Overview, Severity, UNREST_REVOLT};
use l2_kingdom::tables::RATION_NAMES;
use l2_view::Canvas;

use super::*;
use crate::screens::county::county_name;
use crate::shell::{font, Pen};

/// `L2.eng` group 21: the six ration levels, the ration panel's words.
const GROUP_RATION_LEVELS: usize = 21;

/// Each column: its header and the x its numbers end at. The name column starts at [`LEFT`].
const POP: (&str, i32) = ("Pop.", 220);
const HAPPY: (&str, i32) = ("Happy", 290);
const FOOD_X: i32 = 304;
const GRAIN: (&str, i32) = ("Grain", 440);
const CATTLE: (&str, i32) = ("Cattle", 506);
const TAX: (&str, i32) = ("Tax", 560);
const MEN: (&str, i32) = ("Men", RIGHT);

pub(super) fn draw(s: &mut RealmScreen, ctx: &Ctx, canvas: &mut Canvas, o: &Overview, a: &[Alert]) {
    let pen = Pen {
        assets: &ctx.assets.shell,
        ink: &ctx.assets.ink,
        chrome: ctx.assets.chrome.as_ref(),
        shadow: Some(font::SHADOW),
        caps: None,
    };
    let (cols, rows) = match s.view {
        RealmView::Ledger => (BOX_COLS, BOX_ROWS),
        RealmView::EndTurn => (END_COLS, END_ROWS),
    };
    pen.window(canvas, BOX_X, BOX_Y, cols, rows, 0);
    let y = match s.view {
        RealmView::Ledger => {
            pen.heading(canvas, LEFT, BOX_Y + 12, "Realm ledger", font::TEXT);
            let y = table(s, ctx, &pen, canvas, o);
            pen.ok_button(canvas, OK.x, OK.y, 0);
            y
        }
        RealmView::EndTurn => {
            pen.heading(canvas, LEFT, BOX_Y + 12, "End the turn?", font::TEXT);
            gauntlets(&pen, canvas, ctx);
            BOX_Y + 44
        }
    };
    alert_list(s, ctx, &pen, canvas, a, y);
}

fn table(s: &mut RealmScreen, ctx: &Ctx, pen: &Pen, canvas: &mut Canvas, o: &Overview) -> i32 {
    let head = ROW_Y0 - 20;
    pen.body(canvas, LEFT, head, "County", font::TEXT);
    for (label, x) in [POP, HAPPY, GRAIN, CATTLE, TAX, MEN] {
        right(pen, canvas, x, head, label, font::TEXT);
    }
    pen.body(canvas, FOOD_X, head, "Ration", font::TEXT);

    let mut y = ROW_Y0;
    for r in &o.rows {
        row(ctx, pen, canvas, r, y);
        s.hits.push((Rect::new(LEFT, y, RIGHT - LEFT, ROW_H), r.county));
        y += ROW_H;
    }
    canvas.fill_rect(LEFT, y + 1, RIGHT - LEFT, 1, ctx.assets.ink.border);
    y += 5;
    let t = &o.totals;
    pen.body(canvas, LEFT, y, "Total", font::TEXT);
    right(pen, canvas, POP.1, y, &t.population.to_string(), font::TEXT);
    right(pen, canvas, GRAIN.1, y, &t.grain.to_string(), font::TEXT);
    right(pen, canvas, CATTLE.1, y, &t.herd.to_string(), font::TEXT);
    right(pen, canvas, TAX.1, y, &t.tax_expected.to_string(), font::TEXT);
    right(pen, canvas, MEN.1, y, &t.men.to_string(), font::TEXT);
    y += ROW_H + 4;
    let short = t.gold + t.tax_expected < t.wages;
    let line = format!(
        "Treasury {}, tax {}, wages {} crowns.",
        t.gold, t.tax_expected, t.wages
    );
    pen.body(canvas, LEFT, y, &line, if short { font::HIGHLIGHT } else { font::TEXT });
    y + ROW_H + 8
}

fn row(ctx: &Ctx, pen: &Pen, canvas: &mut Canvas, r: &CountyRow, y: i32) {
    let alarm = |bad: bool| if bad { font::HIGHLIGHT } else { font::TEXT };
    let name = county_name(ctx, r.county);
    let name = if r.bridge { format!("{name} *") } else { name };
    pen.body(canvas, LEFT, y, &name, font::TEXT);
    right(pen, canvas, POP.1, y, &r.population.to_string(), font::TEXT);
    let unhappy = r.happiness < l2_kingdom::unrest::HUMAN_WARN_BELOW;
    right(pen, canvas, HAPPY.1, y, &r.happiness.to_string(), alarm(unhappy));
    let short = r.ration_achieved < r.ration_wanted;
    pen.body(canvas, FOOD_X, y, &ration_name(ctx, r.ration_achieved), alarm(short));
    let low = r.grain_seasons().is_some_and(|s| s <= l2_kingdom::overview::GRAIN_SEASONS_LOW);
    right(pen, canvas, GRAIN.1, y, &r.grain.to_string(), alarm(low));
    right(pen, canvas, CATTLE.1, y, &r.herd.to_string(), font::TEXT);
    let tax = if r.tax_suppressed { r.tax_shown } else { r.tax_expected };
    right(pen, canvas, TAX.1, y, &tax.to_string(), alarm(r.tax_suppressed && tax > 0));
    let (men, colour) = if r.enemy_troops > 0 {
        (r.enemy_troops, font::HIGHLIGHT)
    } else {
        (r.friendly_troops, font::TEXT)
    };
    right(pen, canvas, MEN.1, y, &men.to_string(), colour);
}

fn alert_list(s: &mut RealmScreen, ctx: &Ctx, pen: &Pen, canvas: &mut Canvas, a: &[Alert], y: i32) {
    if a.is_empty() {
        pen.body(canvas, LEFT, y, "No alerts.", font::TEXT);
        return;
    }
    let (right, last) = match s.view {
        RealmView::Ledger => (RIGHT, BOTTOM - 28),
        RealmView::EndTurn => (END_RIGHT, END_BOTTOM - 40),
    };
    let room = ((last - y) / ALERT_H).max(1) as usize;
    let shown = if a.len() > room { room - 1 } else { a.len() };
    let mut y = y;
    for alert in &a[..shown] {
        let colour = match alert.severity() {
            Severity::Danger => font::HIGHLIGHT,
            Severity::Warning => font::TEXT,
        };
        pen.body(canvas, LEFT, y, &alert_text(ctx, alert), colour);
        if let Some(county) = alert.county() {
            s.hits.push((Rect::new(LEFT, y, right - LEFT, ALERT_H), county));
        }
        y += ALERT_H;
    }
    if shown < a.len() {
        pen.body(canvas, LEFT, y, &format!("{} more.", a.len() - shown), font::TEXT);
    }
}

pub(super) fn alert_text(ctx: &Ctx, a: &Alert) -> String {
    let name = |c: u8| county_name(ctx, c);
    match *a {
        Alert::Revolt { county, happiness, unrest } => format!(
            "{}: happiness {happiness}, unrest {unrest} of {UNREST_REVOLT}.",
            name(county)
        ),
        Alert::Unhappy { county, happiness } => {
            format!("{}: happiness {happiness}.", name(county))
        }
        Alert::Hungry { county, wanted, achieved } => format!(
            "{}: ration {} next season, not {}.",
            name(county),
            ration_name(ctx, achieved),
            ration_name(ctx, wanted)
        ),
        Alert::GrainLow { county, seasons } => {
            let noun = if seasons == 1 { "season" } else { "seasons" };
            format!("{}: grain for {seasons} {noun}.", name(county))
        }
        Alert::TaxSuppressed { county, shown } => {
            format!("{}: tax of {shown} crowns not collected.", name(county))
        }
        Alert::Wages { gold, income, wages } => {
            format!("Wages {wages} crowns, over treasury {gold} and tax {income}.")
        }
        Alert::Enemy { county, men, bridge } => {
            let cut = if bridge { " Losing it splits the realm." } else { "" };
            format!("{}: {men} enemy men.{cut}", name(county))
        }
    }
}

fn ration_name(ctx: &Ctx, level: i32) -> String {
    let i = level.clamp(0, RATION_NAMES.len() as i32 - 1) as usize;
    let s = ctx.assets.shell.text(GROUP_RATION_LEVELS, i);
    if s.is_empty() { RATION_NAMES[i].to_string() } else { s.to_string() }
}

fn right(pen: &Pen, canvas: &mut Canvas, end: i32, y: i32, s: &str, colour: u8) {
    let w = pen.assets.body.as_ref().map_or(s.len() as i32 * 6, |f| f.width(s));
    pen.body(canvas, end - w, y, s, colour);
}

fn gauntlets(pen: &Pen, canvas: &mut Canvas, ctx: &Ctx) {
    for (r, frame, label) in [(YES, YES_FRAME, "YES"), (NO, NO_FRAME, "NO")] {
        if !pen.chrome.is_some_and(|c| c.draw_system(canvas, frame, r.x, r.y)) {
            crate::widget::button(canvas, &ctx.assets.ink, r, label, false);
        }
    }
}
