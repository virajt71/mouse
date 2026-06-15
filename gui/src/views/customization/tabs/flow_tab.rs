use crate::theme;
use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};
use mouser_engine::config::{Config, FlowPeer};
use mouser_engine::Engine;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlowUiView {
    Welcome,
    SetupWizard,
    Searching,
    NotFound,
}

impl Default for FlowUiView {
    fn default() -> Self {
        FlowUiView::Welcome
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SetupWizardResult {
    None,
    Continue,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NotFoundResult {
    None,
    TryAgain,
    Cancel,
}

pub fn show_flow_tab(ui: &mut egui::Ui, engine: &Engine, config: &mut Config) {
    let mut settings_dirty = false;

    let view_id = ui.make_persistent_id("flow_ui_view");
    let current_view = ui.data_mut(|d| *d.get_temp_mut_or_default::<FlowUiView>(view_id));

    if config.settings.flow_enabled {
        mouser_engine::flow::network::IS_SEARCHING
            .store(false, std::sync::atomic::Ordering::SeqCst);
        show_flow_config_screen(ui, engine, config, &mut settings_dirty);
    } else {
        match current_view {
            FlowUiView::Welcome => {
                mouser_engine::flow::network::IS_SEARCHING
                    .store(false, std::sync::atomic::Ordering::SeqCst);
                if show_flow_welcome_screen(ui) {
                    ui.data_mut(|d| d.insert_temp(view_id, FlowUiView::SetupWizard));
                }
            }
            FlowUiView::SetupWizard => {
                mouser_engine::flow::network::IS_SEARCHING
                    .store(false, std::sync::atomic::Ordering::SeqCst);
                match show_flow_setup_wizard(ui, &config.settings.flow_local_name) {
                    SetupWizardResult::Continue => {
                        ui.data_mut(|d| d.insert_temp(view_id, FlowUiView::Searching));
                    }
                    SetupWizardResult::Cancel => {
                        ui.data_mut(|d| d.insert_temp(view_id, FlowUiView::Welcome));
                    }
                    SetupWizardResult::None => {}
                }
            }
            FlowUiView::Searching => {
                mouser_engine::flow::network::IS_SEARCHING
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                let start_time_id = ui.make_persistent_id("searching_start_time");
                let current_time = ui.ctx().input(|i| i.time);

                let start_time = ui.data_mut(|d| *d.get_temp_mut_or_default::<f64>(start_time_id));

                let start_time = if start_time == 0.0 {
                    ui.data_mut(|d| d.insert_temp(start_time_id, current_time));
                    current_time
                } else {
                    start_time
                };

                let discovered = mouser_engine::flow::network::DISCOVERED_PEERS
                    .read()
                    .unwrap();

                let cancel_clicked =
                    show_flow_searching_screen(ui, &config.settings.flow_local_name);

                if cancel_clicked {
                    mouser_engine::flow::network::IS_SEARCHING
                        .store(false, std::sync::atomic::Ordering::SeqCst);
                    ui.data_mut(|d| d.remove_temp::<f64>(start_time_id));
                    ui.data_mut(|d| d.insert_temp(view_id, FlowUiView::SetupWizard));
                } else if !discovered.is_empty() {
                    mouser_engine::flow::network::IS_SEARCHING
                        .store(false, std::sync::atomic::Ordering::SeqCst);
                    // Auto-pair discovered peers
                    for (name, (ip, _)) in discovered.iter() {
                        if !config.settings.flow_peers.iter().any(|p| p.name == *name) {
                            config.settings.flow_peers.push(FlowPeer {
                                name: name.clone(),
                                ip: ip.clone(),
                                port: 50520,
                                layout_x: 1,
                                layout_y: 0,
                                paired: true,
                                fingerprint: "".to_string(),
                                auto_reconnect: true,
                            });
                        }
                    }
                    ui.data_mut(|d| d.remove_temp::<f64>(start_time_id));
                    config.settings.flow_enabled = true;
                    settings_dirty = true;
                    ui.data_mut(|d| d.insert_temp(view_id, FlowUiView::Welcome));
                } else if current_time - start_time > 45.0 {
                    mouser_engine::flow::network::IS_SEARCHING
                        .store(false, std::sync::atomic::Ordering::SeqCst);
                    ui.data_mut(|d| d.remove_temp::<f64>(start_time_id));
                    ui.data_mut(|d| d.insert_temp(view_id, FlowUiView::NotFound));
                }
            }
            FlowUiView::NotFound => {
                mouser_engine::flow::network::IS_SEARCHING
                    .store(false, std::sync::atomic::Ordering::SeqCst);
                match show_flow_not_found_screen(ui, &config.settings.flow_local_name) {
                    NotFoundResult::TryAgain => {
                        ui.data_mut(|d| d.insert_temp(view_id, FlowUiView::Searching));
                    }
                    NotFoundResult::Cancel => {
                        ui.data_mut(|d| d.insert_temp(view_id, FlowUiView::Welcome));
                    }
                    NotFoundResult::None => {}
                }
            }
        }
    }

    if settings_dirty {
        let _ = config.save();
        engine.increment_config_generation();
    }
}

fn draw_welcome_illustration(ui: &mut egui::Ui) {
    let desired_size = vec2(400.0, 180.0);
    let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let c = rect.center();
    let time = ui.ctx().input(|i| i.time);

    ui.ctx().request_repaint();

    // 1. Draw circular ring loop
    let ring_center = c + vec2(0.0, -10.0);
    let ring_radius = 75.0;
    painter.circle_stroke(
        ring_center,
        ring_radius,
        Stroke::new(1.8, Color32::from_gray(60)),
    );

    // 2. Left Laptop
    let left_center = ring_center - vec2(80.0, -10.0);
    painter.rect_filled(
        Rect::from_center_size(left_center + vec2(0.0, 31.0), vec2(110.0, 4.0)),
        2.0,
        Color32::from_rgb(0x4E, 0x51, 0x66),
    );
    let kb_rect = Rect::from_center_size(left_center + vec2(0.0, 28.0), vec2(100.0, 4.0));
    painter.rect_filled(kb_rect, 1.0, Color32::from_rgb(0x3B, 0x3E, 0x52));

    let screen_frame_rect = Rect::from_center_size(left_center + vec2(0.0, 2.0), vec2(90.0, 56.0));
    painter.rect_filled(screen_frame_rect, 4.0, Color32::from_rgb(0x1E, 0x20, 0x2C));
    painter.rect_stroke(
        screen_frame_rect,
        4.0,
        Stroke::new(1.0, Color32::from_gray(80)),
    );

    let display_rect = Rect::from_center_size(left_center + vec2(0.0, 1.0), vec2(84.0, 48.0));
    painter.rect_filled(display_rect, 2.0, Color32::from_rgb(0x3F, 0x00, 0xB5));

    let file1 = Rect::from_min_size(display_rect.min + vec2(8.0, 8.0), vec2(6.0, 8.0));
    painter.rect_filled(file1, 0.0, Color32::from_rgb(0xF5, 0xC2, 0x42));
    let file2 = Rect::from_min_size(display_rect.min + vec2(8.0, 20.0), vec2(6.0, 8.0));
    painter.rect_filled(file2, 0.0, Color32::WHITE);

    // 3. Right Laptop
    let right_center = ring_center + vec2(80.0, 10.0);
    painter.rect_filled(
        Rect::from_center_size(right_center + vec2(0.0, 31.0), vec2(110.0, 4.0)),
        2.0,
        Color32::from_rgb(0x4E, 0x51, 0x66),
    );
    let kb_rect_r = Rect::from_center_size(right_center + vec2(0.0, 28.0), vec2(100.0, 4.0));
    painter.rect_filled(kb_rect_r, 1.0, Color32::from_rgb(0x3B, 0x3E, 0x52));

    let screen_frame_rect_r =
        Rect::from_center_size(right_center + vec2(0.0, 2.0), vec2(90.0, 56.0));
    painter.rect_filled(
        screen_frame_rect_r,
        4.0,
        Color32::from_rgb(0x1E, 0x20, 0x2C),
    );
    painter.rect_stroke(
        screen_frame_rect_r,
        4.0,
        Stroke::new(1.0, Color32::from_gray(80)),
    );

    let display_rect_r = Rect::from_center_size(right_center + vec2(0.0, 1.0), vec2(84.0, 48.0));
    painter.rect_filled(display_rect_r, 2.0, Color32::from_rgb(0x0C, 0x1B, 0x3A));
    painter.rect_filled(
        Rect::from_min_max(
            display_rect_r.left_top() + vec2(42.0, 0.0),
            display_rect_r.right_bottom(),
        ),
        0.0,
        Color32::from_rgb(0xFF, 0x3B, 0x5C),
    );

    let file1_r = Rect::from_min_size(display_rect_r.min + vec2(8.0, 8.0), vec2(6.0, 8.0));
    painter.rect_filled(file1_r, 0.0, Color32::from_rgb(0x00, 0xE5, 0xFF));
    let file2_r = Rect::from_min_size(display_rect_r.min + vec2(8.0, 20.0), vec2(6.0, 8.0));
    painter.rect_filled(file2_r, 0.0, Color32::WHITE);

    // 4. Plant
    let plant_center = ring_center + vec2(0.0, 20.0);
    let pot_rect = Rect::from_center_size(plant_center + vec2(0.0, 10.0), vec2(10.0, 14.0));
    painter.rect_filled(pot_rect, 1.0, Color32::from_rgb(0x9E, 0x9E, 0x9E));
    let leaf_color = Color32::from_rgb(0x00, 0xE3, 0xC5);
    let sway = (time * 2.2).sin() * 1.2;
    painter.line_segment(
        [
            plant_center + vec2(0.0, 3.0),
            plant_center + vec2(-4.0 + sway as f32, -6.0),
        ],
        Stroke::new(2.5, leaf_color),
    );
    painter.line_segment(
        [
            plant_center + vec2(0.0, 3.0),
            plant_center + vec2(4.0 + sway as f32, -6.0),
        ],
        Stroke::new(2.5, leaf_color),
    );
    painter.line_segment(
        [
            plant_center + vec2(0.0, 3.0),
            plant_center + vec2(sway as f32 * 0.5, -9.0),
        ],
        Stroke::new(2.5, leaf_color),
    );

    // 5. Star Cursor Glow
    let star_center = display_rect_r.center() + vec2(-10.0, 5.0);
    let star_color = Color32::WHITE;
    let pulse_scale = 1.0 + (time * 4.5).sin() * 0.15;
    for i in 0..8 {
        let angle = (i as f32) * std::f32::consts::PI / 4.0;
        let length = 6.0 * pulse_scale;
        let dx = angle.cos() * length as f32;
        let dy = angle.sin() * length as f32;
        painter.line_segment(
            [star_center, star_center + vec2(dx, dy)],
            Stroke::new(1.5, star_color),
        );
    }
    let pointer_points = vec![
        star_center + vec2(4.0, 4.0),
        star_center + vec2(12.0, 8.0),
        star_center + vec2(9.0, 9.0),
        star_center + vec2(12.0, 14.0),
        star_center + vec2(10.0, 15.0),
        star_center + vec2(7.0, 10.0),
        star_center + vec2(5.0, 12.0),
    ];
    painter.add(egui::Shape::convex_polygon(
        pointer_points,
        Color32::from_rgb(0x11, 0x11, 0x11),
        Stroke::new(1.0, Color32::WHITE),
    ));

    // 6. Floating particles
    let float_y = (time * 1.5).sin() * 3.0;
    let float_x = (time * 1.2).cos() * 2.0;
    painter.circle_filled(
        ring_center + vec2(-110.0, -35.0 + float_y as f32),
        3.5,
        Color32::from_rgb(0xFF, 0x4A, 0x4A),
    );
    painter.circle_filled(
        ring_center + vec2(-35.0 + float_x as f32, -65.0),
        3.0,
        Color32::from_rgb(0xFF, 0xB7, 0x4D),
    );
    painter.circle_filled(
        ring_center + vec2(45.0, 55.0 - float_y as f32),
        2.5,
        Color32::from_rgb(0x00, 0xE3, 0xC5),
    );
    painter.circle_filled(
        ring_center + vec2(110.0 + float_x as f32, 15.0 + float_y as f32),
        4.0,
        Color32::from_rgb(0xF8, 0xBB, 0xD0),
    );
}

fn show_flow_welcome_screen(ui: &mut egui::Ui) -> bool {
    let mut setup_flow_clicked = false;
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);

        draw_welcome_illustration(ui);

        ui.add_space(30.0);

        ui.add(egui::Label::new(
            RichText::new("Welcome to Logi Flow")
                .color(Color32::WHITE)
                .size(26.0)
                .strong(),
        ));

        ui.add_space(15.0);

        let desc_text = "Use and control multiple computers seamlessly with Flow. Switch to another computer by simply moving your cursor to the edge of the screen. Need to transfer text, images, or files between computers? Just copy them on one machine and paste on the other.";
        ui.set_max_width(520.0);
        let mut job = egui::text::LayoutJob::default();
        job.halign = egui::Align::Center;
        job.append(
            desc_text,
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(13.0),
                color: theme::secondary_text(ui.ctx()),
                ..Default::default()
            }
        );
        ui.add(egui::Label::new(job).wrap());

        ui.add_space(35.0);

        let btn_id = ui.make_persistent_id("setup_flow_button");
        let btn_res = ui.allocate_response(vec2(160.0, 40.0), egui::Sense::click());
        let is_hovered = btn_res.hovered();
        
        let t = ui.ctx().animate_bool(btn_id, is_hovered);
        
        let fill_color = theme::lerp_color(
            Color32::from_rgb(0x00, 0xF3, 0xC5),
            Color32::from_rgb(0x33, 0xFF, 0xD7),
            t
        );

        let stroke_color = theme::lerp_color(
            Color32::TRANSPARENT,
            Color32::WHITE,
            t
        );

        ui.painter().rect_filled(btn_res.rect, 4.0, fill_color);
        if t > 0.0 {
            ui.painter().rect_stroke(btn_res.rect, 4.0, Stroke::new(1.0 * t, stroke_color));
        }

        let btn_text = RichText::new("SETUP FLOW")
            .color(Color32::BLACK)
            .size(13.0)
            .strong();

        ui.painter().text(
            btn_res.rect.center(),
            egui::Align2::CENTER_CENTER,
            btn_text.text(),
            egui::FontId::proportional(13.0),
            Color32::BLACK,
        );

        if btn_res.clicked() {
            setup_flow_clicked = true;
        }

        ui.add_space(40.0);
    });
    setup_flow_clicked
}

fn draw_setup_cards(ui: &mut egui::Ui, local_name: &str) {
    let desired_size = vec2(500.0, 250.0);
    let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let c = rect.center() + vec2(0.0, 35.0);

    let card_w = 180.0;
    let card_h = 110.0;

    let left_center = c - vec2(110.0, 0.0);
    let right_center = c + vec2(110.0, 0.0);

    let left_rect = Rect::from_center_size(left_center, vec2(card_w, card_h));
    let right_rect = Rect::from_center_size(right_center, vec2(card_w, card_h));

    // 1. Draw connecting line
    painter.line_segment(
        [left_rect.right_center(), right_rect.left_center()],
        Stroke::new(1.8, Color32::from_gray(50)),
    );

    // 2. Draw Left Card (Local Computer)
    painter.rect_filled(left_rect, 6.0, Color32::from_rgb(0x07, 0x07, 0x08));
    painter.rect_stroke(left_rect, 6.0, Stroke::new(1.2, Color32::from_gray(65)));

    painter.text(
        left_center - vec2(0.0, 10.0),
        egui::Align2::CENTER_CENTER,
        local_name,
        egui::FontId::proportional(15.0),
        Color32::WHITE,
    );

    let pill_rect = Rect::from_center_size(left_center + vec2(0.0, 18.0), vec2(56.0, 20.0));
    painter.rect_filled(
        pill_rect,
        3.0,
        Color32::from_rgba_unmultiplied(0, 227, 197, 20),
    );
    painter.rect_stroke(
        pill_rect,
        3.0,
        Stroke::new(1.0, Color32::from_rgb(0x00, 0xE3, 0xC5)),
    );
    painter.text(
        pill_rect.center(),
        egui::Align2::CENTER_CENTER,
        "READY",
        egui::FontId::proportional(9.0),
        Color32::from_rgb(0x00, 0xE3, 0xC5),
    );

    // 3. Draw Right Card (Other computer(s))
    painter.rect_filled(
        right_rect,
        6.0,
        Color32::from_rgba_unmultiplied(40, 40, 40, 120),
    );
    painter.rect_stroke(right_rect, 6.0, Stroke::new(1.0, Color32::from_gray(45)));

    painter.text(
        right_center,
        egui::Align2::CENTER_CENTER,
        "Other computer(s)",
        egui::FontId::proportional(15.0),
        Color32::from_gray(200),
    );

    // 4. Draw Tooltip Box above Right Card
    let tooltip_w = 205.0;
    let tooltip_h = 70.0;
    let tooltip_center = right_center - vec2(0.0, 105.0);
    let tooltip_rect = Rect::from_center_size(tooltip_center, vec2(tooltip_w, tooltip_h));

    painter.rect_filled(tooltip_rect, 6.0, Color32::BLACK);
    painter.rect_stroke(tooltip_rect, 6.0, Stroke::new(1.2, Color32::from_gray(55)));

    // Little triangle pointing down
    let tooltip_bottom_center = pos2(tooltip_rect.center().x, tooltip_rect.bottom());
    let tri_p1 = tooltip_bottom_center + vec2(-6.0, 0.0);
    let tri_p2 = tooltip_bottom_center + vec2(6.0, 0.0);
    let tri_p3 = tooltip_bottom_center + vec2(0.0, 6.0);
    painter.add(egui::Shape::convex_polygon(
        vec![tri_p1, tri_p2, tri_p3],
        Color32::BLACK,
        Stroke::new(1.2, Color32::from_gray(55)),
    ));
    painter.line_segment([tri_p1, tri_p2], Stroke::new(1.5, Color32::BLACK));

    let bullets = [
        "1. Install Mouser-RS",
        "2. Pair mouse on different channel",
        "3. Connect to same network",
    ];

    for (i, text) in bullets.iter().enumerate() {
        let pos = tooltip_rect.left_top() + vec2(14.0, 14.0 + (i as f32 * 17.0));
        painter.text(
            pos,
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::proportional(10.5),
            Color32::WHITE,
        );
    }
}

fn show_flow_setup_wizard(ui: &mut egui::Ui, local_name: &str) -> SetupWizardResult {
    let mut wizard_result = SetupWizardResult::None;
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);

        draw_setup_cards(ui, local_name);

        ui.add_space(40.0);

        ui.add(egui::Label::new(
            RichText::new("Connect other computer(s)")
                .color(Color32::WHITE)
                .size(26.0)
                .strong(),
        ));

        ui.add_space(15.0);

        ui.set_max_width(520.0);
        let mut job = egui::text::LayoutJob::default();
        job.halign = egui::Align::Center;

        job.append(
            "Follow the above 3 steps on other computers to connect to them via Flow. ",
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(13.0),
                color: theme::secondary_text(ui.ctx()),
                ..Default::default()
            },
        );

        job.append(
            "Need help?",
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(13.0),
                color: Color32::from_rgb(0x00, 0xE3, 0xC5),
                ..Default::default()
            },
        );

        let label_res = ui.add(egui::Label::new(job).wrap().sense(egui::Sense::click()));
        if label_res.clicked() {
            // Open help
        }

        ui.add_space(35.0);

        // Button: CONTINUE
        let btn_continue_id = ui.make_persistent_id("continue_setup_btn");
        let btn_continue_res = ui.allocate_response(vec2(180.0, 40.0), egui::Sense::click());
        let is_hovered_c = btn_continue_res.hovered();
        let t_c = ui.ctx().animate_bool(btn_continue_id, is_hovered_c);
        let fill_color_c = theme::lerp_color(
            Color32::from_rgb(0x00, 0xF3, 0xC5),
            Color32::from_rgb(0x33, 0xFF, 0xD7),
            t_c,
        );
        let stroke_color_c = theme::lerp_color(Color32::TRANSPARENT, Color32::WHITE, t_c);
        ui.painter()
            .rect_filled(btn_continue_res.rect, 4.0, fill_color_c);
        if t_c > 0.0 {
            ui.painter().rect_stroke(
                btn_continue_res.rect,
                4.0,
                Stroke::new(1.0 * t_c, stroke_color_c),
            );
        }
        ui.painter().text(
            btn_continue_res.rect.center(),
            egui::Align2::CENTER_CENTER,
            "CONTINUE",
            egui::FontId::proportional(13.0),
            Color32::BLACK,
        );

        if btn_continue_res.clicked() {
            wizard_result = SetupWizardResult::Continue;
        }

        ui.add_space(12.0);

        // Button: CANCEL
        let btn_cancel_id = ui.make_persistent_id("cancel_setup_btn");
        let btn_cancel_res = ui.allocate_response(vec2(180.0, 40.0), egui::Sense::click());
        let is_hovered_can = btn_cancel_res.hovered();
        let t_can = ui.ctx().animate_bool(btn_cancel_id, is_hovered_can);
        let fill_color_can = theme::lerp_color(
            Color32::from_rgb(0x0B, 0x0B, 0x0B),
            Color32::from_rgb(0x18, 0x18, 0x18),
            t_can,
        );
        let stroke_color_can = theme::lerp_color(
            Color32::from_gray(80),
            Color32::from_rgb(0x00, 0xF3, 0xC5),
            t_can,
        );
        ui.painter()
            .rect_filled(btn_cancel_res.rect, 4.0, fill_color_can);
        ui.painter()
            .rect_stroke(btn_cancel_res.rect, 4.0, Stroke::new(1.2, stroke_color_can));
        ui.painter().text(
            btn_cancel_res.rect.center(),
            egui::Align2::CENTER_CENTER,
            "CANCEL",
            egui::FontId::proportional(13.0),
            Color32::from_rgb(0x00, 0xF3, 0xC5),
        );

        if btn_cancel_res.clicked() {
            wizard_result = SetupWizardResult::Cancel;
        }

        ui.add_space(40.0);
    });
    wizard_result
}

fn draw_searching_cards(ui: &mut egui::Ui, local_name: &str) {
    let desired_size = vec2(500.0, 250.0);
    let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let c = rect.center() + vec2(0.0, 35.0);
    let time = ui.ctx().input(|i| i.time);
    ui.ctx().request_repaint();

    let card_w = 180.0;
    let card_h = 110.0;

    let left_center = c - vec2(110.0, 0.0);
    let right_center = c + vec2(110.0, 0.0);

    let left_rect = Rect::from_center_size(left_center, vec2(card_w, card_h));
    let right_rect = Rect::from_center_size(right_center, vec2(card_w, card_h));

    // 1. Draw connecting line
    painter.line_segment(
        [left_rect.right_center(), right_rect.left_center()],
        Stroke::new(1.5, Color32::from_rgb(0x2A, 0x2A, 0x2B)),
    );

    // 2. Draw Left Card (Local Computer)
    painter.rect_filled(left_rect, 6.0, Color32::from_rgb(0x06, 0x06, 0x06));
    painter.rect_stroke(
        left_rect,
        6.0,
        Stroke::new(1.0, Color32::from_rgb(0x2A, 0x2A, 0x2B)),
    );

    painter.text(
        left_center - vec2(0.0, 10.0),
        egui::Align2::CENTER_CENTER,
        local_name,
        egui::FontId::proportional(22.0),
        Color32::WHITE,
    );

    let pill_rect = Rect::from_center_size(left_center + vec2(0.0, 22.0), vec2(58.0, 20.0));
    painter.rect_filled(pill_rect, 3.0, Color32::from_rgb(0x06, 0x06, 0x06));
    painter.rect_stroke(
        pill_rect,
        3.0,
        Stroke::new(1.0, Color32::from_rgb(0x00, 0xE3, 0xC5)),
    );
    painter.text(
        pill_rect.center(),
        egui::Align2::CENTER_CENTER,
        "READY",
        egui::FontId::proportional(9.0),
        Color32::from_rgb(0x00, 0xE3, 0xC5),
    );

    // 3. Draw Right Card (Searching Card with rotating spinner)
    painter.rect_filled(right_rect, 6.0, Color32::from_rgb(0x06, 0x06, 0x06));
    painter.rect_stroke(
        right_rect,
        6.0,
        Stroke::new(1.0, Color32::from_rgb(0x2A, 0x2A, 0x2B)),
    );

    // Draw circular rotating spinner with a fading tail
    let spinner_radius = 20.0;
    let num_spinner_segments = 40;
    let arc_length = 1.5 * std::f32::consts::PI; // 270 degrees
    let base_angle = (time * 6.0) as f32; // speed of rotation

    for i in 0..num_spinner_segments {
        let fraction = i as f32 / (num_spinner_segments as f32);
        let angle = base_angle + fraction * arc_length;

        let alpha_u8 = (fraction * 255.0) as u8;
        let color = Color32::from_rgba_unmultiplied(0, 227, 197, alpha_u8);

        let next_angle = base_angle + ((i + 1) as f32 / (num_spinner_segments as f32)) * arc_length;
        let start = right_center + vec2(angle.cos() * spinner_radius, angle.sin() * spinner_radius);
        let end = right_center
            + vec2(
                next_angle.cos() * spinner_radius,
                next_angle.sin() * spinner_radius,
            );

        painter.line_segment([start, end], Stroke::new(2.5, color));
    }
}

fn show_flow_searching_screen(ui: &mut egui::Ui, local_name: &str) -> bool {
    let mut cancel_clicked = false;
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);

        draw_searching_cards(ui, local_name);

        ui.add_space(30.0);

        ui.add(egui::Label::new(
            RichText::new("Searching for computers")
                .color(Color32::WHITE)
                .size(32.0)
                .strong(),
        ));

        ui.add_space(12.0);

        ui.add(egui::Label::new(
            RichText::new("This process may take up to a minute")
                .color(theme::secondary_text(ui.ctx()))
                .size(14.0),
        ));

        ui.add_space(30.0);

        // Button: CANCEL
        let btn_cancel_id = ui.make_persistent_id("cancel_searching_btn");
        let btn_cancel_res = ui.allocate_response(vec2(140.0, 46.0), egui::Sense::click());
        let is_hovered_can = btn_cancel_res.hovered();
        let t_can = ui.ctx().animate_bool(btn_cancel_id, is_hovered_can);

        let fill_color_can = Color32::from_rgb(0x06, 0x06, 0x06);
        let stroke_color_can = theme::lerp_color(
            Color32::from_rgb(0x2A, 0x2A, 0x2B),
            Color32::from_rgb(0x00, 0xE3, 0xC5),
            t_can,
        );

        ui.painter()
            .rect_filled(btn_cancel_res.rect, 6.0, fill_color_can);
        ui.painter()
            .rect_stroke(btn_cancel_res.rect, 6.0, Stroke::new(1.0, stroke_color_can));

        ui.painter().text(
            btn_cancel_res.rect.center(),
            egui::Align2::CENTER_CENTER,
            "CANCEL",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(0x00, 0xE3, 0xC5),
        );

        if btn_cancel_res.clicked() {
            cancel_clicked = true;
        }

        ui.add_space(40.0);
    });
    cancel_clicked
}

fn draw_not_found_cards(ui: &mut egui::Ui, local_name: &str) {
    let desired_size = vec2(500.0, 250.0);
    let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let c = rect.center() + vec2(0.0, 35.0);

    let card_w = 180.0;
    let card_h = 110.0;

    let left_center = c - vec2(110.0, 0.0);
    let right_center = c + vec2(110.0, 0.0);

    let left_rect = Rect::from_center_size(left_center, vec2(card_w, card_h));
    let right_rect = Rect::from_center_size(right_center, vec2(card_w, card_h));

    // 1. Draw connecting line
    painter.line_segment(
        [left_rect.right_center(), right_rect.left_center()],
        Stroke::new(1.5, Color32::from_rgb(0x2A, 0x2A, 0x2B)),
    );

    // 2. Draw Left Card (Local Computer)
    painter.rect_filled(left_rect, 6.0, Color32::from_rgb(0x06, 0x06, 0x06));
    painter.rect_stroke(
        left_rect,
        6.0,
        Stroke::new(1.0, Color32::from_rgb(0x2A, 0x2A, 0x2B)),
    );

    painter.text(
        left_center - vec2(0.0, 10.0),
        egui::Align2::CENTER_CENTER,
        local_name,
        egui::FontId::proportional(22.0),
        Color32::WHITE,
    );

    let pill_rect = Rect::from_center_size(left_center + vec2(0.0, 22.0), vec2(58.0, 20.0));
    painter.rect_filled(pill_rect, 3.0, Color32::from_rgb(0x06, 0x06, 0x06));
    painter.rect_stroke(
        pill_rect,
        3.0,
        Stroke::new(1.0, Color32::from_rgb(0x00, 0xE3, 0xC5)),
    );
    painter.text(
        pill_rect.center(),
        egui::Align2::CENTER_CENTER,
        "READY",
        egui::FontId::proportional(9.0),
        Color32::from_rgb(0x00, 0xE3, 0xC5),
    );

    // 3. Draw Right Card (Not Found Card with orange/amber outline)
    painter.rect_filled(right_rect, 6.0, Color32::from_rgb(0x06, 0x06, 0x06));
    painter.rect_stroke(
        right_rect,
        6.0,
        Stroke::new(1.2, Color32::from_rgb(0xE5, 0x8C, 0x0D)),
    );

    // Draw orange circular filled warning icon with exclamation point
    let warning_center = right_center - vec2(0.0, 10.0);
    painter.circle_filled(warning_center, 18.0, Color32::from_rgb(0xE5, 0x8C, 0x0D));
    painter.text(
        warning_center + vec2(0.0, 1.0),
        egui::Align2::CENTER_CENTER,
        "!",
        egui::FontId::proportional(22.0),
        Color32::WHITE,
    );

    // Draw orange "NOT FOUND" pill badge below
    let not_found_rect = Rect::from_center_size(right_center + vec2(0.0, 24.0), vec2(84.0, 20.0));
    painter.rect_filled(not_found_rect, 3.0, Color32::from_rgb(0xE5, 0x8C, 0x0D));
    painter.text(
        not_found_rect.center() + vec2(0.0, 1.0),
        egui::Align2::CENTER_CENTER,
        "NOT FOUND",
        egui::FontId::proportional(9.0),
        Color32::BLACK,
    );

    // 4. Draw Tooltip Box above Right Card (shifted slightly right)
    let tooltip_w = 205.0;
    let tooltip_h = 70.0;
    let tooltip_center = right_center - vec2(-15.0, 105.0);
    let tooltip_rect = Rect::from_center_size(tooltip_center, vec2(tooltip_w, tooltip_h));

    painter.rect_filled(tooltip_rect, 6.0, Color32::BLACK);
    painter.rect_stroke(tooltip_rect, 6.0, Stroke::new(1.2, Color32::from_gray(55)));

    // Left-pointing triangle arrow on the left side of the tooltip
    let tooltip_left_center = tooltip_rect.left_center();
    let tri_p1 = tooltip_left_center + vec2(0.0, -6.0);
    let tri_p2 = tooltip_left_center + vec2(0.0, 6.0);
    let tri_p3 = tooltip_left_center - vec2(6.0, 0.0);
    painter.add(egui::Shape::convex_polygon(
        vec![tri_p1, tri_p2, tri_p3],
        Color32::BLACK,
        Stroke::new(1.2, Color32::from_gray(55)),
    ));
    // Merge arrow base line
    painter.line_segment([tri_p1, tri_p2], Stroke::new(1.5, Color32::BLACK));

    let bullets = [
        "1. Install Mouser-RS",
        "2. Pair mouse on different channel",
        "3. Connect to same network",
    ];

    for (i, text) in bullets.iter().enumerate() {
        let pos = tooltip_rect.left_top() + vec2(14.0, 14.0 + (i as f32 * 17.0));
        painter.text(
            pos,
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::proportional(10.5),
            Color32::WHITE,
        );
    }
}

fn show_flow_not_found_screen(ui: &mut egui::Ui, local_name: &str) -> NotFoundResult {
    let mut result = NotFoundResult::None;
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);

        draw_not_found_cards(ui, local_name);

        ui.add_space(30.0);

        ui.add(egui::Label::new(
            RichText::new("No computers found. Bummer!")
                .color(Color32::WHITE)
                .size(32.0)
                .strong(),
        ));

        ui.add_space(15.0);

        // Center aligned layout job for the description text with inline link
        ui.set_max_width(520.0);
        let mut job = egui::text::LayoutJob::default();
        job.halign = egui::Align::Center;
        
        job.append(
            "We searched everywhere... No computers found on your network. Let's try again—follow the above steps on other computers. ",
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(14.0),
                color: theme::secondary_text(ui.ctx()),
                ..Default::default()
            }
        );
        
        job.append(
            "Need help?",
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(14.0),
                color: Color32::from_rgb(0x00, 0xE3, 0xC5),
                ..Default::default()
            }
        );
        
        let label_res = ui.add(egui::Label::new(job).wrap().sense(egui::Sense::click()));
        if label_res.clicked() {
            // Help link clicked
        }

        ui.add_space(30.0);

        // Button: TRY AGAIN
        let btn_try_id = ui.make_persistent_id("try_again_btn");
        let btn_try_res = ui.allocate_response(vec2(180.0, 40.0), egui::Sense::click());
        let is_hovered_t = btn_try_res.hovered();
        let t_t = ui.ctx().animate_bool(btn_try_id, is_hovered_t);
        let fill_color_t = theme::lerp_color(
            Color32::from_rgb(0x00, 0xE3, 0xC5),
            Color32::from_rgb(0x33, 0xFF, 0xD7),
            t_t
        );
        let stroke_color_t = theme::lerp_color(
            Color32::TRANSPARENT,
            Color32::WHITE,
            t_t
        );
        ui.painter().rect_filled(btn_try_res.rect, 4.0, fill_color_t);
        if t_t > 0.0 {
            ui.painter().rect_stroke(btn_try_res.rect, 4.0, Stroke::new(1.0 * t_t, stroke_color_t));
        }
        ui.painter().text(
            btn_try_res.rect.center(),
            egui::Align2::CENTER_CENTER,
            "TRY AGAIN",
            egui::FontId::proportional(13.0),
            Color32::BLACK,
        );

        if btn_try_res.clicked() {
            result = NotFoundResult::TryAgain;
        }

        ui.add_space(12.0);

        // Button: CANCEL
        let btn_cancel_id = ui.make_persistent_id("cancel_not_found_btn");
        let btn_cancel_res = ui.allocate_response(vec2(180.0, 40.0), egui::Sense::click());
        let is_hovered_can = btn_cancel_res.hovered();
        let t_can = ui.ctx().animate_bool(btn_cancel_id, is_hovered_can);
        
        let fill_color_can = Color32::from_rgb(0x06, 0x06, 0x06);
        let stroke_color_can = theme::lerp_color(
            Color32::from_rgb(0x2A, 0x2A, 0x2B),
            Color32::from_rgb(0x00, 0xE3, 0xC5),
            t_can
        );
        
        ui.painter().rect_filled(btn_cancel_res.rect, 6.0, fill_color_can);
        ui.painter().rect_stroke(btn_cancel_res.rect, 6.0, Stroke::new(1.0, stroke_color_can));
        
        ui.painter().text(
            btn_cancel_res.rect.center(),
            egui::Align2::CENTER_CENTER,
            "CANCEL",
            egui::FontId::proportional(13.0),
            Color32::from_rgb(0x00, 0xE3, 0xC5),
        );

        if btn_cancel_res.clicked() {
            result = NotFoundResult::Cancel;
        }

        ui.add_space(40.0);
    });
    result
}

fn show_flow_config_screen(
    ui: &mut egui::Ui,
    _engine: &Engine,
    config: &mut Config,
    settings_dirty: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.add_space(20.0);
        ui.vertical(|ui| {
            ui.add_space(20.0);
            
            // Header
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(
                    RichText::new("LOGITECH FLOW")
                        .color(Color32::WHITE)
                        .size(18.0)
                        .strong(),
                ));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut enabled = config.settings.flow_enabled;
                    let response = ui.allocate_response(vec2(40.0, 20.0), egui::Sense::click());
                    if response.clicked() {
                        enabled = !enabled;
                        config.settings.flow_enabled = enabled;
                        *settings_dirty = true;
                    }
                    let t = ui.ctx().animate_bool(response.id.with("flow_enable_toggle"), enabled);
                    let color = theme::lerp_color(Color32::from_gray(50), theme::accent_color(ui.ctx()), t);
                    ui.painter().rect_filled(response.rect, 10.0, color);
                    let knob_x = response.rect.min.x + 10.0 + t * (response.rect.width() - 20.0);
                    ui.painter().circle_filled(pos2(knob_x, response.rect.center().y), 8.0, Color32::WHITE);
                });
            });

            ui.add_space(8.0);
            ui.add(egui::Label::new(
                RichText::new("Seamlessly control multiple computers, copy and paste files, and sync clipboards over your local network.")
                    .color(theme::secondary_text(ui.ctx()))
                    .size(12.0),
            ));
            
            ui.add_space(20.0);

            // Two-column layout: Left is layout grid, Right is Settings / Peers
            ui.columns(2, |columns| {
                // Column 0: Visual Screen Grid Layout Configurator
                let ui_grid = &mut columns[0];
                ui_grid.vertical(|ui| {
                    ui.label(
                        RichText::new("MONITOR ARRANGEMENT")
                            .color(theme::primary_text(ui.ctx()))
                            .size(11.0)
                            .strong(),
                    );
                    ui.add_space(10.0);

                    // Draw a 3x3 layout grid representing monitors
                    let grid_center = ui.cursor().left_top() + vec2(150.0, 100.0);
                    let box_w = 90.0;
                    let box_h = 60.0;

                    let mut clicked_slot = None;

                    let mut draw_screen = |ui: &mut egui::Ui, offset_x: i32, offset_y: i32, label: &str, is_local: bool| {
                        let center_pos = grid_center + vec2(offset_x as f32 * (box_w + 10.0), offset_y as f32 * (box_h + 10.0));
                        let r = Rect::from_center_size(center_pos, vec2(box_w, box_h));
                        let res = ui.allocate_rect(r, egui::Sense::click());

                        let bg = if is_local {
                            theme::accent_dim_color(ui.ctx())
                        } else if res.hovered() {
                            theme::hover_color(ui.ctx())
                        } else {
                            theme::surface_color(ui.ctx())
                        };

                        let border = if is_local {
                            theme::accent_color(ui.ctx())
                        } else if !label.is_empty() {
                            theme::accent_color(ui.ctx())
                        } else {
                            theme::border_color(ui.ctx())
                        };

                        ui.painter().rect_filled(r, 4.0, bg);
                        ui.painter().rect_stroke(r, 4.0, Stroke::new(1.2, border));
                        theme::draw_tech_corners(ui.painter(), r, border, 5.0);

                        let display_label = if label.is_empty() {
                            if is_local { config.settings.flow_local_name.clone() } else { "[Empty]".to_string() }
                        } else {
                            label.to_string()
                        };

                        ui.painter().text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            &display_label,
                            egui::FontId::proportional(10.0),
                            theme::primary_text(ui.ctx()),
                        );

                        if !is_local && res.clicked() {
                            clicked_slot = Some((offset_x, offset_y));
                        }
                    };

                    // Render 3x3 layout cross
                    draw_screen(ui, 0, 0, "", true);

                    let mut left_peer = "".to_string();
                    let mut right_peer = "".to_string();
                    let mut top_peer = "".to_string();
                    let mut bottom_peer = "".to_string();

                    for peer in &config.settings.flow_peers {
                        if peer.layout_x == -1 && peer.layout_y == 0 { left_peer = peer.name.clone(); }
                        else if peer.layout_x == 1 && peer.layout_y == 0 { right_peer = peer.name.clone(); }
                        else if peer.layout_x == 0 && peer.layout_y == -1 { top_peer = peer.name.clone(); }
                        else if peer.layout_x == 0 && peer.layout_y == 1 { bottom_peer = peer.name.clone(); }
                    }

                    draw_screen(ui, -1, 0, &left_peer, false);
                    draw_screen(ui, 1, 0, &right_peer, false);
                    draw_screen(ui, 0, -1, &top_peer, false);
                    draw_screen(ui, 0, 1, &bottom_peer, false);

                    if let Some((offset_x, offset_y)) = clicked_slot {
                        let mut available = vec!["".to_string()];
                        for peer in &config.settings.flow_peers {
                            if peer.paired {
                                available.push(peer.name.clone());
                            }
                        }
                        
                        let current = config.settings.flow_peers.iter()
                            .find(|p| p.layout_x == offset_x && p.layout_y == offset_y)
                            .map(|p| p.name.clone())
                            .unwrap_or_default();
                        
                        let current_idx = available.iter().position(|x| x == &current).unwrap_or(0);
                        let next_idx = (current_idx + 1) % available.len();
                        let new_peer_name = &available[next_idx];
                        
                        for p in &mut config.settings.flow_peers {
                            if p.layout_x == offset_x && p.layout_y == offset_y {
                                p.layout_x = 99;
                                p.layout_y = 99;
                            }
                        }
                        
                        if !new_peer_name.is_empty() {
                            for p in &mut config.settings.flow_peers {
                                if p.name == *new_peer_name {
                                    p.layout_x = offset_x;
                                    p.layout_y = offset_y;
                                }
                            }
                        }
                        *settings_dirty = true;
                    }

                    ui.add_space(200.0);
                });

                // Column 1: Config Parameters
                let ui_params = &mut columns[1];
                ui_params.vertical(|ui| {
                    ui.label(
                        RichText::new("FLOW SETTINGS")
                            .color(theme::primary_text(ui.ctx()))
                            .size(11.0)
                            .strong(),
                    );
                    ui.add_space(10.0);

                    // Local Name Input
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("This Computer:").color(theme::secondary_text(ui.ctx())).size(11.0));
                        let mut local_name = config.settings.flow_local_name.clone();
                        if ui.text_edit_singleline(&mut local_name).changed() {
                            config.settings.flow_local_name = local_name;
                            *settings_dirty = true;
                        }
                    });
                    ui.add_space(8.0);

                    // Mouse Mode Dropdown/Combo
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Switching Method:").color(theme::secondary_text(ui.ctx())).size(11.0));
                        let current_mode = config.settings.flow_mouse_mode.clone();
                        egui::ComboBox::from_id_salt("mouse_redirection_mode_combobox")
                            .selected_text(if current_mode == "hardware" { "Hardware (HID++ Channel Switch)" } else { "Software Redirection (Instant)" })
                            .show_ui(ui, |ui| {
                                if ui.selectable_value(&mut config.settings.flow_mouse_mode, "software".to_string(), "Software Redirection (Instant)").clicked() {
                                    *settings_dirty = true;
                                }
                                if ui.selectable_value(&mut config.settings.flow_mouse_mode, "hardware".to_string(), "Hardware (HID++ Channel Switch)").clicked() {
                                    *settings_dirty = true;
                                }
                            });
                    });
                    ui.add_space(8.0);

                    // Hold key Dropdown/Combo
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Hold Key to Transition:").color(theme::secondary_text(ui.ctx())).size(11.0));
                        let hold_key = config.settings.flow_hold_key.clone();
                        egui::ComboBox::from_id_salt("hold_key_combobox")
                            .selected_text(hold_key.to_uppercase())
                            .show_ui(ui, |ui| {
                                if ui.selectable_value(&mut config.settings.flow_hold_key, "none".to_string(), "NONE").clicked() {
                                    *settings_dirty = true;
                                }
                                if ui.selectable_value(&mut config.settings.flow_hold_key, "ctrl".to_string(), "CTRL").clicked() {
                                    *settings_dirty = true;
                                }
                                if ui.selectable_value(&mut config.settings.flow_hold_key, "alt".to_string(), "ALT").clicked() {
                                    *settings_dirty = true;
                                }
                                if ui.selectable_value(&mut config.settings.flow_hold_key, "shift".to_string(), "SHIFT").clicked() {
                                    *settings_dirty = true;
                                }
                            });
                    });
                    ui.add_space(8.0);

                    // Keyboard Linking checkbox
                    ui.checkbox(&mut config.settings.flow_keyboard_linking, "Link keyboard input redirection");
                    ui.add_space(10.0);

                    // Resolution inputs for Wayland coordinate accumulation
                    ui.label(
                        RichText::new("Screen Resolution (required for Wayland)")
                            .color(theme::secondary_text(ui.ctx()))
                            .size(10.5),
                    );
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        let mut w_str = config.settings.flow_screen_width.to_string();
                        if ui.add(egui::TextEdit::singleline(&mut w_str).desired_width(50.0)).changed() {
                            if let Ok(w) = w_str.parse::<i32>() {
                                config.settings.flow_screen_width = w;
                                *settings_dirty = true;
                            }
                        }
                        ui.label("Height:");
                        let mut h_str = config.settings.flow_screen_height.to_string();
                        if ui.add(egui::TextEdit::singleline(&mut h_str).desired_width(50.0)).changed() {
                            if let Ok(h) = h_str.parse::<i32>() {
                                config.settings.flow_screen_height = h;
                                *settings_dirty = true;
                            }
                        }
                    });

                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("DISCOVERED COMPUTERS")
                            .color(theme::primary_text(ui.ctx()))
                            .size(11.0)
                            .strong(),
                    );
                    ui.add_space(8.0);

                    // Display list of discovered peers from network
                    let discovered = mouser_engine::flow::network::DISCOVERED_PEERS.read().unwrap();
                    if discovered.is_empty() {
                        ui.label(RichText::new("No computers found on local subnet. Make sure they are running Mouser-RS and connected to the same network.").color(theme::muted_text(ui.ctx())).size(10.5));
                    } else {
                        for (name, (ip, _)) in discovered.iter() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("{} ({})", name, ip)).color(theme::primary_text(ui.ctx())).size(11.0));
                                
                                // Check if this peer is already paired
                                let paired = config.settings.flow_peers.iter().any(|p| p.name == *name && p.paired);
                                if paired {
                                    ui.label(RichText::new("Paired").color(theme::accent_color(ui.ctx())).size(10.5));
                                } else {
                                    if ui.button("Pair").clicked() {
                                        config.settings.flow_peers.push(FlowPeer {
                                            name: name.clone(),
                                            ip: ip.clone(),
                                            port: 50520,
                                            layout_x: 1,
                                            layout_y: 0,
                                            paired: true,
                                            fingerprint: "".to_string(),
                                            auto_reconnect: true,
                                        });
                                        *settings_dirty = true;
                                    }
                                }
                            });
                        }
                    }
                });
            });
        });
    });
}
