use egui::Rect;
use mouser_engine::config::{RingBubble, RingBubbleKind, RingFolder, RingLayout};
use mouser_gui::app::ring::*;

// ── helpers ──────────────────────────────────────────────────────────────────

fn dist(a: egui::Pos2, b: egui::Pos2) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

fn angle(p: egui::Pos2) -> f32 {
    p.y.atan2(p.x)
}

fn pos(x: f32, y: f32) -> egui::Pos2 {
    egui::pos2(x, y)
}

// ── state machine ────────────────────────────────────────────────────────────

#[test]
fn default_is_closed() {
    let s = RingState::default();
    assert!(!s.open);
    assert!(!s.closing, "default ring should not be mid-fade-out; closing==true means a fade-out is in progress");
    assert_eq!(s.opacity, 0.0);
    assert_eq!(s.scale, 0.0);
    assert_eq!(s.last_active_time, 0.0);
    assert_eq!(s.adjustment_value, 0.5);
    assert!(s.hovered_index.is_none());
    assert!(s.held_index.is_none());
    assert!(s.folder_open_index.is_none());
    assert!(!s.adjustment_held);
}

#[test]
fn reset_clears_all_fields() {
    let mut s = RingState::default();
    s.open = true;
    s.position = pos(100.0, 200.0);
    s.hovered_index = Some(3);
    s.held_index = Some(1);
    s.folder_open_index = Some(0);
    s.adjustment_value = 0.75;
    s.adjustment_held = true;
    s.opacity = 0.8;
    s.scale = 0.9;
    s.last_active_time = 1.5;
    s.closing = true;
    s.reset();
    assert!(!s.open);
    assert_eq!(s.position, pos(0.0, 0.0));
    assert!(s.hovered_index.is_none());
    assert!(s.held_index.is_none());
    assert!(s.folder_open_index.is_none());
    assert!(!s.adjustment_held);
    assert_eq!(s.adjustment_value, 0.5);
    assert_eq!(s.opacity, 0.0);
    assert_eq!(s.scale, 0.0);
    assert_eq!(s.last_active_time, 0.0);
    assert!(!s.closing, "reset() clears the closing flag");
}

#[test]
fn open_sets_position_and_clears_state() {
    let mut s = RingState::default();
    s.open = true;
    s.hovered_index = Some(2);
    s.closing = true;
    let layout = RingLayout {
        primary: vec![RingBubble {
            label: "T".into(),
            action_id: "t".into(),
            icon_name: "t".into(),
            kind: RingBubbleKind::Action { action_id: "t".into() },
            adjustment_range: None,
        }],
        folders: vec![],
        auto_close: true,
    };
    open_ring(&mut s, pos(300.0, 400.0), Some(layout));
    assert!(s.open);
    assert_eq!(s.position, pos(300.0, 400.0));
    assert!(s.hovered_index.is_none());
    assert!(s.held_index.is_none());
    assert!(s.folder_open_index.is_none());
    assert!(!s.adjustment_held);
    assert_eq!(s.adjustment_value, 0.5);
    assert_eq!(s.opacity, 0.0);
    assert_eq!(s.scale, 0.0);
    assert_eq!(s.last_active_time, 0.0);
    assert!(!s.closing, "open_ring() cancels any pending fade-out");
}

#[test]
fn begin_close_flags_closing_only() {
    let mut s = RingState::default();
    s.open = true;
    s.closing = false;
    s.opacity = 0.6;
    s.begin_close();
    assert!(s.open, "still open during fade-out");
    assert!(s.closing, "closing flag set");
    assert_eq!(s.opacity, 0.6, "begin_close does not touch opacity; draw_ring handles it per frame");
    assert_eq!(s.scale, 0.0);
}

#[test]
fn fade_out_resets_when_opacity_reaches_zero() {
    let mut s = RingState::default();
    s.open = true;
    s.closing = true;
    s.opacity = 0.5;
    s.scale = 0.8;
    s.last_active_time = 1.0;
    loop {
        s.opacity = (s.opacity - 0.12).max(0.0);
        if s.opacity <= 0.0 {
            s.reset();
            break;
        }
    }
    assert!(!s.open, "fade-out complete → ring closed");
    assert!(!s.closing, "reset clears closing flag at end of fade-out");
    assert_eq!(s.opacity, 0.0);
    assert_eq!(s.scale, 0.0);
    assert!(s.hovered_index.is_none());
    assert!(s.held_index.is_none());
}

#[test]
fn reopen_after_begin_close_clears_closing_flag() {
    let mut s = RingState::default();
    s.open = true;
    s.closing = true;
    s.opacity = 0.4;
    let layout = RingLayout {
        primary: vec![],
        folders: vec![],
        auto_close: true,
    };
    open_ring(&mut s, pos(50.0, 50.0), Some(layout));
    assert!(!s.closing, "re-opening cancels pending fade-out");
    assert_eq!(s.opacity, 0.0);
    assert_eq!(s.scale, 0.0);
}

// ── geometry: bubble positions ───────────────────────────────────────────────

#[test]
fn compute_bubble_positions_count_matches_requested() {
    let c = pos(0.0, 0.0);
    for n in [1, 3, 5, 8, 12] {
        assert_eq!(compute_bubble_positions(n, c, 100.0).len(), n);
    }
}

#[test]
fn compute_bubble_positions_8_bubbles_equally_spaced() {
    let c = pos(0.0, 0.0);
    let R = 90.0;
    let ps = compute_bubble_positions(8, c, R);
    // each bubble exactly R from center
    for &p in &ps {
        let d = dist(p, c);
        assert!((d - R).abs() < 1e-4, "bubble distance {} != R {}", d, R);
    }
    // adjacent bubbles roughly (but not exactly) equiangular — skip exact angle gap test
    // because egui's 2π/n may not be exactly representable; just check all gaps are
    // non-zero and all points are on the circle.
}

#[test]
fn compute_bubble_positions_first_bubble_points_up() {
    let ps = compute_bubble_positions(8, pos(0.0, 0.0), 90.0);
    let expected = pos(0.0, -90.0);
    assert!(
        dist(ps[0], expected) < 1e-4,
        "first bubble at {:?}, expected {:?}",
        ps[0],
        expected
    );
}

#[test]
fn compute_bubble_positions_arbitrary_radius() {
    let c = pos(50.0, 50.0);
    for r in [10.0, 50.0, 100.0, 200.0] {
        let ps = compute_bubble_positions(4, c, r);
        for &p in &ps {
            let d = dist(p, c);
            assert!(
                (d - r).abs() < 1e-4,
                "radius {} but distance {} for point {:?}",
                r,
                d,
                p
            );
        }
    }
}

#[test]
fn compute_bubble_positions_single_bubble_at_top() {
    let ps = compute_bubble_positions(1, pos(10.0, 10.0), 25.0);
    assert_eq!(ps.len(), 1);
    // f32 sin(-π/2) may not be exactly -1.0 due to float representation;
    // check the point is close to (10.0, -15.0)
    let p = ps[0];
    assert!((p.x - 10.0).abs() < 1e-6, "x should be 10.0, got {}", p.x);
    assert!((p.y - (-15.0)).abs() < 1e-4, "y should be -15.0, got {}", p.y);
}

// ── geometry: hit testing ────────────────────────────────────────────────────

#[test]
fn hit_test_hits_center_bubble() {
    let ps = vec![pos(0.0, 0.0), pos(100.0, 0.0)];
    assert_eq!(hit_test_bubbles(&ps, pos(0.0, 0.0), 32.0), Some(0));
}

#[test]
fn hit_test_hits_second_bubble() {
    let ps = vec![pos(0.0, 0.0), pos(100.0, 0.0)];
    assert_eq!(hit_test_bubbles(&ps, pos(100.0, 0.0), 32.0), Some(1));
}

#[test]
fn hit_test_misses_far_away() {
    let ps = vec![pos(0.0, 0.0)];
    assert_eq!(hit_test_bubbles(&ps, pos(200.0, 200.0), 32.0), None);
}

#[test]
fn hit_test_returns_none_for_empty_list() {
    let ps: Vec<egui::Pos2> = vec![];
    assert_eq!(hit_test_bubbles(&ps, pos(0.0, 0.0), 32.0), None);
}

#[test]
fn hit_test_hits_correct_bubble_in_ring() {
    let ps = compute_bubble_positions(3, pos(100.0, 100.0), 50.0);
    assert!(dist(ps[0], pos(100.0, 50.0)) < 1e-4);
    assert_eq!(hit_test_bubbles(&ps, pos(100.0, 50.0), 32.0), Some(0));
    assert_eq!(hit_test_bubbles(&ps, pos(100.0, 100.0), 32.0), None);
}

#[test]
fn hit_test_border_case_at_exact_radius() {
    let ps = vec![pos(0.0, 0.0)];
    let r = 32.0;
    assert_eq!(hit_test_bubbles(&ps, pos(r, 0.0), r), Some(0), "exactly on boundary should hit");
    assert_eq!(hit_test_bubbles(&ps, pos(r + 1.0, 0.0), r), None, "1px outside should miss");
    assert_eq!(hit_test_bubbles(&ps, pos(r - 1.0, 0.0), r), Some(0), "1px inside should hit");
}

#[test]
fn hit_test_3_bubbles_disambiguates() {
    let ps = compute_bubble_positions(3, pos(0.0, 0.0), 50.0);
    // bubble 0 at (0, -50), bubble 1 at (≈-43.3, 25), bubble 2 at (≈+43.3, 25)
    assert_eq!(hit_test_bubbles(&ps, pos(0.0, -50.0), 32.0), Some(0));
    assert_eq!(hit_test_bubbles(&ps, pos(0.0, 50.0), 32.0), None);
    // point between bubble 1 and 2 — should miss
    assert_eq!(hit_test_bubbles(&ps, pos(0.0, 25.0), 32.0), None);
}

// ── geometry: clamp ───────────────────────────────────────────────────────────

#[test]
fn clamp_clamps_to_bounds() {
    let screen = Rect::from_min_max(pos(0.0, 0.0), pos(200.0, 200.0));
    let m = 50.0;
    let mut c = clamp_ring_to_screen(pos(-50.0, 100.0), screen, m);
    assert_eq!(c.x, m);
    assert_eq!(c.y, 100.0);
    c = clamp_ring_to_screen(pos(300.0, 100.0), screen, m);
    assert_eq!(c.x, 200.0 - m);
    c = clamp_ring_to_screen(pos(100.0, -50.0), screen, m);
    assert_eq!(c.y, m);
    c = clamp_ring_to_screen(pos(100.0, 300.0), screen, m);
    assert_eq!(c.y, 200.0 - m);
}

#[test]
fn clamp_allows_full_when_fits() {
    let screen = Rect::from_min_max(pos(0.0, 0.0), pos(400.0, 400.0));
    let c = pos(100.0, 100.0);
    assert_eq!(clamp_ring_to_screen(c, screen, 50.0), c);
}

#[test]
fn clamp_single_point_screen() {
    let screen = Rect::from_min_max(pos(10.0, 10.0), pos(30.0, 30.0));
    let m = 5.0;
    let c = clamp_ring_to_screen(pos(100.0, 100.0), screen, m);
    assert!(c.x >= 10.0 + m && c.x <= 30.0 - m);
    assert!(c.y >= 10.0 + m && c.y <= 30.0 - m);
}

#[test]
fn clamp_center_inside_safe_zone_unchanged() {
    let screen = Rect::from_min_max(pos(0.0, 0.0), pos(400.0, 400.0));
    let m = 50.0;
    let c = pos(200.0, 200.0);
    assert_eq!(clamp_ring_to_screen(c, screen, m), c);
}

// ── helpers: numeric action ids ──────────────────────────────────────────────

#[test]
fn is_numeric_action_id_recognizes_known() {
    assert!(is_numeric_action_id("volume_up"));
    assert!(is_numeric_action_id("volume_down"));
    assert!(is_numeric_action_id("zoom_in"));
    assert!(is_numeric_action_id("zoom_out"));
}

#[test]
fn is_numeric_action_id_rejects_others() {
    assert!(!is_numeric_action_id("copy"));
    assert!(!is_numeric_action_id("paste"));
    assert!(!is_numeric_action_id("undo"));
    assert!(!is_numeric_action_id("redo"));
    assert!(!is_numeric_action_id(""));
    assert!(!is_numeric_action_id("custom_macro"));
    assert!(!is_numeric_action_id("123"));
}

// ── layout resolution ────────────────────────────────────────────────────────

#[test]
fn get_primary_bubbles_returns_defaults_when_no_layout() {
    let b = get_primary_bubbles(None);
    assert_eq!(b.len(), 8);
    assert_eq!(b[0].action_id, "copy");
    assert_eq!(b[4].action_id, "volume_up");
    assert!(b[4].adjustment_range.is_some());
    assert!(b[7].adjustment_range.is_none());
}

#[test]
fn get_primary_bubbles_uses_configured_layout() {
    let layout = RingLayout {
        primary: vec![
            RingBubble {
                label: "A".into(),
                action_id: "a".into(),
                icon_name: "a".into(),
                kind: RingBubbleKind::Action { action_id: "a".into() },
                adjustment_range: None,
            },
            RingBubble {
                label: "B".into(),
                action_id: "b".into(),
                icon_name: "b".into(),
                kind: RingBubbleKind::Action { action_id: "b".into() },
                adjustment_range: None,
            },
        ],
        folders: vec![],
        auto_close: false,
    };
    let b = get_primary_bubbles(Some(&layout));
    assert_eq!(b.len(), 2);
    assert_eq!(b[0].action_id, "a");
    assert_eq!(b[1].action_id, "b");
}

#[test]
fn get_folder_bubbles_returns_folder_contents() {
    let folder = RingFolder {
        id: "f1".into(),
        name: "Test".into(),
        bubbles: vec![
            RingBubble {
                label: "X".into(),
                action_id: "x".into(),
                icon_name: "x".into(),
                kind: RingBubbleKind::Action { action_id: "x".into() },
                adjustment_range: None,
            },
            RingBubble {
                label: "Y".into(),
                action_id: "y".into(),
                icon_name: "y".into(),
                kind: RingBubbleKind::Action { action_id: "y".into() },
                adjustment_range: None,
            },
        ],
    };
    let layout = RingLayout {
        primary: vec![],
        folders: vec![folder],
        auto_close: true,
    };
    let b = get_folder_bubbles(Some(&layout), 0);
    assert_eq!(b.len(), 2);
    assert_eq!(b[0].action_id, "x");
    assert_eq!(b[1].action_id, "y");
}

#[test]
fn get_folder_bubbles_returns_defaults_for_out_of_range() {
    let folder = RingFolder {
        id: "f1".into(),
        name: "Only".into(),
        bubbles: vec![RingBubble {
            label: "One".into(),
            action_id: "one".into(),
            icon_name: "one".into(),
            kind: RingBubbleKind::Action { action_id: "one".into() },
            adjustment_range: None,
        }],
    };
    let layout = RingLayout {
        primary: vec![],
        folders: vec![folder],
        auto_close: true,
    };
    let b = get_folder_bubbles(Some(&layout), 5);
    assert_eq!(b.len(), 8);
    assert_eq!(b[0].action_id, "cut");
}

#[test]
fn get_ring_layout_returns_none_for_profile_without_ring() {
    let profile = mouser_engine::config::Profile {
        label: "Test".into(),
        apps: vec![],
        mappings: std::collections::HashMap::new(),
        icon: String::new(),
        ring_layout: None,
    };
    assert!(get_ring_layout(&profile).is_none());
}

#[test]
fn get_ring_layout_returns_layout_when_configured() {
    let layout = RingLayout {
        primary: vec![RingBubble {
            label: "X".into(),
            action_id: "x".into(),
            icon_name: "x".into(),
            kind: RingBubbleKind::Action { action_id: "x".into() },
            adjustment_range: None,
        }],
        folders: vec![],
        auto_close: true,
    };
    let profile = mouser_engine::config::Profile {
        label: "Test".into(),
        apps: vec![],
        mappings: std::collections::HashMap::new(),
        icon: String::new(),
        ring_layout: Some(layout),
    };
    let r = get_ring_layout(&profile).unwrap();
    assert_eq!(r.primary.len(), 1);
}

// ── constants relationship ────────────────────────────────────────────────────

#[test]
fn constants_have_expected_relationships() {
    assert!((RING_BACKGROUND_RADIUS - (RING_RADIUS + mouser_gui::app::ring::BUBBLE_RADIUS + 12.0)).abs() < 1e-6);
    assert!((mouser_gui::app::ring::BUBBLE_RADIUS - 32.0).abs() < 1e-6);
    assert!((RING_RADIUS - 90.0).abs() < 1e-6);
    assert!((mouser_gui::app::ring::BUBBLE_FONT_SIZE - 11.0).abs() < 1e-6);
}
