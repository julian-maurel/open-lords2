use super::*;

fn window() -> Rect {
    Rect::new(BOX_X, BOX_Y, BOX_COLS * 16, BOX_ROWS * 16)
}

fn inside(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.w <= outer.x + outer.w
        && inner.y + inner.h <= outer.y + outer.h
}

#[test]
fn the_window_is_on_screen_under_the_menu_bar() {
    let w = window();
    assert!(w.x >= 0 && w.y >= 24, "under the menu bar");
    assert!(w.x + w.w <= 640 && w.y + w.h <= 480);
}

#[test]
fn every_button_is_inside_its_window_and_the_two_gauntlets_do_not_overlap() {
    assert!(inside(window(), OK), "{OK:?}");
    let end = Rect::new(BOX_X, BOX_Y, END_COLS * 16, END_ROWS * 16);
    assert!(end.x + end.w <= l2_view::campaign::PANEL_X, "the right column stays in sight");
    for b in [YES, NO] {
        assert!(inside(end, b), "{b:?}");
    }
    assert!(YES.x + YES.w <= NO.x);
}

#[test]
fn sixteen_counties_leave_room_for_the_totals_and_an_alert() {
    let after_rows = ROW_Y0 + (l2_kingdom::county::MAX_COUNTIES as i32 - 1) * ROW_H;
    let after_totals = after_rows + 5 + ROW_H + 4 + ROW_H + 8;
    assert!(after_totals + ALERT_H <= BOTTOM - 28, "{after_totals}");
}
