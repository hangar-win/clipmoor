#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

const GAP: i32 = 4;

#[derive(serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum Align {
    #[default]
    Start,
    Center,
}

pub fn scaled((w, h): (f64, f64), dpi: u32) -> (i32, i32) {
    let s = dpi.max(96) as f64 / 96.0;
    ((w * s).round() as i32, (h * s).round() as i32)
}

pub fn place(anchor: Rect, (w, h): (i32, i32), work: Rect, align: Align) -> (i32, i32) {
    let left = match align {
        Align::Start => anchor.left,
        Align::Center => anchor.left + (anchor.right - anchor.left) / 2 - w / 2,
    };
    let x = left.min(work.right - w).max(work.left);
    let below = anchor.bottom + GAP;
    let y = if below + h <= work.bottom { below } else { (anchor.top - GAP - h).max(work.top) };
    (x, y)
}

pub fn resized(old: Rect, (w, _): (i32, i32), align: Align) -> (i32, i32) {
    match align {
        Align::Start => (old.left, old.top),
        Align::Center => (old.left + (old.right - old.left) / 2 - w / 2, old.top),
    }
}

pub fn center((w, h): (i32, i32), area: Rect, work: Rect) -> (i32, i32) {
    let x = area.left + (area.right - area.left - w) / 2;
    let y = area.top + (area.bottom - area.top - h) / 2;
    clamp((x, y), (w, h), work)
}

pub fn clamp((x, y): (i32, i32), (w, h): (i32, i32), work: Rect) -> (i32, i32) {
    (x.min(work.right - w).max(work.left), y.min(work.bottom - h).max(work.top))
}

pub fn centered((w, h): (i32, i32), min: i32, margin: i32, over: Rect, work: Rect) -> Rect {
    let room = Rect { left: work.left + margin, top: work.top + margin, right: work.right - margin, bottom: work.bottom - margin };
    let (room_w, room_h) = (room.right - room.left, room.bottom - room.top);
    let s = (room_w as f64 / w.max(1) as f64).min(room_h as f64 / h.max(1) as f64).min(1.0);
    let w = ((w as f64 * s).round() as i32).max(min).min(room_w);
    let h = ((h as f64 * s).round() as i32).max(min).min(room_h);
    let (x, y) = center((w, h), over, room);
    Rect { left: x, top: y, right: x + w, bottom: y + h }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Gti,
    Msaa,
    Uia,
    Mouse,
    Center,
}

pub fn is_caret(r: Rect, dpi: u32, screen: Rect) -> bool {
    let h = r.bottom - r.top;
    h > 0 && h <= 64 * dpi as i32 / 96 && r.right - r.left <= (screen.right - screen.left) / 2
}

pub fn anchor(found: Option<(Method, Rect)>, mouse: Option<Rect>, target: Rect, dpi: u32, screen: Rect) -> (Method, Rect) {
    match (found, mouse) {
        (Some((m, r)), _) if is_caret(r, dpi, screen) => (m, r),
        (_, Some(p)) if p.left >= target.left && p.left < target.right && p.top >= target.top && p.top < target.bottom => (Method::Mouse, p),
        _ => (Method::Center, target),
    }
}

pub fn spot((method, a): (Method, Rect), size: (i32, i32), work: Rect, align: Align) -> (i32, i32) {
    if method == Method::Center { center(size, a, work) } else { place(a, size, work, align) }
}

#[cfg(test)]
mod tests {
    use super::*;
    const WORK: Rect = Rect { left: 0, top: 0, right: 1920, bottom: 1040 };
    fn at(x: i32, y: i32) -> Rect { Rect { left: x, top: y, right: x + 1, bottom: y + 18 } }

    #[test]
    fn below_caret_when_it_fits() {
        assert_eq!(place(at(100, 100), (448, 640), WORK, Align::Start), (100, 122));
    }

    #[test]
    fn above_caret_near_bottom() {
        assert_eq!(place(at(100, 900), (448, 640), WORK, Align::Start), (100, 256));
    }

    #[test]
    fn clamped_to_right_and_top_edges() {
        assert_eq!(place(at(1800, 100), (448, 640), WORK, Align::Start), (1472, 122));
        assert_eq!(place(at(100, 500), (448, 640), WORK, Align::Start).1, 0);
    }

    #[test]
    fn clamp_keeps_an_on_screen_spot() {
        assert_eq!(clamp((300, 200), (448, 640), WORK), (300, 200));
    }

    #[test]
    fn clamp_pulls_an_overhanging_panel_back_onto_the_work_area() {
        assert_eq!(clamp((1700, 900), (448, 640), WORK), (1472, 400));
        assert_eq!(clamp((-120, -50), (448, 640), WORK), (0, 0));
    }

    #[test]
    fn clamp_on_a_left_monitor_with_negative_x() {
        let left = Rect { left: -2560, top: 0, right: 0, bottom: 1400 };
        assert_eq!(clamp((-2700, 100), (448, 640), left), (-2560, 100));
        assert_eq!(clamp((-300, 1000), (448, 640), left), (-448, 760));
    }

    fn r(left: i32, top: i32, w: i32, h: i32) -> Rect { Rect { left, top, right: left + w, bottom: top + h } }

    #[test]
    fn preview_sits_in_the_middle_of_the_work_area() {
        assert_eq!(centered((900, 700), 640, 40, WORK, WORK), r(510, 170, 900, 700));
    }

    #[test]
    fn big_preview_shrinks_to_the_work_area_keeping_aspect() {
        assert_eq!(centered((3000, 2000), 640, 40, WORK, WORK), r(240, 40, 1440, 960));
    }

    #[test]
    fn small_preview_keeps_the_minimum() {
        assert_eq!(centered((200, 100), 640, 40, WORK, WORK), r(640, 200, 640, 640));
    }

    #[test]
    fn preview_centred_on_a_negative_x_monitor() {
        let left = Rect { left: -1920, top: -200, right: 0, bottom: 880 };
        assert_eq!(centered((800, 600), 640, 40, left, left), r(-1360, 20, 800, 640));
    }

    #[test]
    fn preview_centres_over_the_panel_and_stays_inside_the_margin() {
        let panel = |left: i32| Rect { left, top: 200, right: left + 448, bottom: 840 };
        assert_eq!(centered((640, 480), 0, 40, panel(600), WORK), r(504, 280, 640, 480));
        assert_eq!(centered((640, 480), 0, 40, panel(20), WORK), r(40, 280, 640, 480));
    }

    #[test]
    fn center_of_work_area() {
        assert_eq!(center((448, 640), WORK, WORK), (736, 200));
    }

    #[test]
    fn center_of_small_window_stays_on_screen() {
        let small = Rect { left: 1700, top: 900, right: 1900, bottom: 1000 };
        assert_eq!(center((448, 640), small, WORK), (1472, 400));
    }

    const WIN: Rect = Rect { left: 200, top: 100, right: 1600, bottom: 1000 };
    fn pt(x: i32, y: i32) -> Rect { Rect { left: x, top: y, right: x + 1, bottom: y + 1 } }

    #[test]
    fn caret_sized_rects_are_carets() {
        assert!(is_caret(at(100, 100), 96, WORK));
        assert!(is_caret(Rect { left: 10, top: 10, right: 12, bottom: 50 }, 192, WORK));
        assert!(is_caret(Rect { left: 10, top: 10, right: 900, bottom: 40 }, 96, WORK));
    }

    #[test]
    fn editor_or_page_sized_rects_are_not_carets() {
        assert!(!is_caret(Rect { left: 300, top: 150, right: 1500, bottom: 950 }, 96, WORK));
        assert!(!is_caret(Rect { left: 10, top: 10, right: 12, bottom: 90 }, 96, WORK));
        assert!(!is_caret(Rect { left: 0, top: 10, right: 1000, bottom: 30 }, 96, WORK));
        assert!(!is_caret(Rect { left: 10, top: 10, right: 11, bottom: 10 }, 96, WORK));
    }

    #[test]
    fn real_caret_wins() {
        assert_eq!(anchor(Some((Method::Msaa, at(400, 300))), Some(pt(50, 50)), WIN, 96, WORK), (Method::Msaa, at(400, 300)));
    }

    #[test]
    fn big_rect_falls_back_to_mouse_inside_target() {
        let editor = Rect { left: 300, top: 150, right: 1500, bottom: 950 };
        assert_eq!(anchor(Some((Method::Uia, editor)), Some(pt(700, 500)), WIN, 96, WORK), (Method::Mouse, pt(700, 500)));
        assert_eq!(anchor(None, Some(pt(700, 500)), WIN, 96, WORK), (Method::Mouse, pt(700, 500)));
    }

    #[test]
    fn mouse_outside_target_falls_back_to_window_center() {
        let editor = Rect { left: 300, top: 150, right: 1500, bottom: 950 };
        assert_eq!(anchor(Some((Method::Uia, editor)), Some(pt(1700, 500)), WIN, 96, WORK), (Method::Center, WIN));
        assert_eq!(anchor(None, None, WIN, 96, WORK), (Method::Center, WIN));
    }

    const LEFT_WORK: Rect = Rect { left: -2560, top: 0, right: 0, bottom: 1400 };
    const DESKTOP: Rect = Rect { left: -2560, top: 0, right: 2560, bottom: 1440 };

    #[test]
    fn desktop_on_left_monitor_follows_the_mouse_not_the_focused_icon() {
        let icon = Rect { left: -2560, top: 5, right: -2485, bottom: 81 };
        assert!(!is_caret(icon, 96, LEFT_WORK));
        let a = anchor(None, Some(pt(-1171, 638)), DESKTOP, 96, LEFT_WORK);
        assert_eq!(a, (Method::Mouse, pt(-1171, 638)));
        assert_eq!(spot(a, (448, 640), LEFT_WORK, Align::Start), (-1171, 643));
    }

    #[test]
    fn negative_work_areas_keep_the_panel_on_that_monitor() {
        assert_eq!(spot((Method::Mouse, pt(-2550, 1300)), (448, 640), LEFT_WORK, Align::Start), (-2550, 656));
        assert_eq!(spot((Method::Mouse, pt(-10, 10)), (448, 640), LEFT_WORK, Align::Start), (-448, 15));
        let above = Rect { left: 0, top: -1080, right: 1920, bottom: -40 };
        assert_eq!(spot((Method::Msaa, at(300, -200)), (448, 640), above, Align::Start), (300, -844));
        assert_eq!(spot((Method::Center, Rect { left: 0, top: -1080, right: 1920, bottom: 0 }), (448, 640), above, Align::Start), (736, -860));
    }

    #[test]
    fn centred_placement_puts_the_middle_of_the_window_under_the_caret() {
        assert_eq!(place(at(1000, 100), (920, 260), WORK, Align::Center), (540, 122));
        assert_eq!(spot((Method::Mouse, pt(1000, 100)), (920, 260), WORK, Align::Center), (540, 105));
        assert_eq!(spot((Method::Center, WIN), (920, 260), WORK, Align::Center), spot((Method::Center, WIN), (920, 260), WORK, Align::Start));
    }

    #[test]
    fn centred_placement_near_an_edge_stays_on_that_monitor() {
        assert_eq!(place(at(100, 100), (920, 260), WORK, Align::Center), (0, 122));
        assert_eq!(place(at(1900, 900), (920, 260), WORK, Align::Center), (1000, 636));
        assert_eq!(place(at(-2500, 300), (920, 260), LEFT_WORK, Align::Center), (-2560, 322));
        assert_eq!(place(at(-30, 300), (920, 260), LEFT_WORK, Align::Center), (-920, 322));
    }

    #[test]
    fn resizing_keeps_the_left_edge_or_the_horizontal_centre() {
        let old = r(540, 122, 920, 260);
        assert_eq!(resized(old, (630, 260), Align::Start), (540, 122));
        assert_eq!(resized(old, (630, 260), Align::Center), (685, 122));
        assert_eq!(resized(old, (630, 48), Align::Center), (685, 122));
        let narrow = r(685, 122, 630, 260);
        assert_eq!(resized(narrow, (920, 260), Align::Center), (540, 122));
        assert_eq!(resized(r(-1000, 5, 920, 260), (631, 260), Align::Center), (-855, 5));
    }

    #[test]
    fn logical_sizes_scale_with_the_monitor_dpi() {
        assert_eq!(scaled((448.0, 640.0), 96), (448, 640));
        assert_eq!(scaled((920.0, 260.0), 144), (1380, 390));
        assert_eq!(scaled((448.0, 48.0), 120), (560, 60));
        assert_eq!(scaled((680.0, 420.0), 0), (680, 420));
    }

    #[test]
    fn spot_puts_panel_under_caret_or_in_window_center() {
        assert_eq!(spot((Method::Gti, at(100, 100)), (448, 640), WORK, Align::Start), (100, 122));
        assert_eq!(spot((Method::Mouse, pt(100, 100)), (448, 640), WORK, Align::Start), (100, 105));
        assert_eq!(spot((Method::Center, WIN), (448, 640), WORK, Align::Start), (676, 230));
    }
}
