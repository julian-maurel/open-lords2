use super::*;

fn window() -> Rect {
    Rect::new(BOX_X, BOX_Y, BOX_COLS * 16, BOX_ROWS * 16)
}

#[test]
fn the_window_is_on_screen_and_every_line_and_the_ok_button_are_inside_it() {
    let w = window();
    assert!(w.x >= 0 && w.y >= 24, "under the menu bar");
    assert!(w.x + w.w <= 640 && w.y + w.h <= 480);
    let inside = |r: Rect| r.x >= w.x && r.y >= w.y && r.x + r.w <= w.x + w.w && r.y + r.h <= w.y + w.h;
    assert!(inside(OK));
    for c in Cheat::LIST {
        assert!(inside(row(c)), "{c:?}");
    }
}

#[test]
fn the_lines_do_not_overlap_and_leave_room_for_the_stockpiles_above_ok() {
    for pair in Cheat::LIST.windows(2) {
        assert!(row(pair[0]).y + ROW_H <= row(pair[1]).y);
    }
    let last = row(Cheat::Everything);
    let after = last.y + last.h + 6 + 3 * ROW_H;
    assert!(after <= OK.y, "{after}");
}
