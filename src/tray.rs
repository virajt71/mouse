use eframe::egui;
use engine::Engine;

pub fn setup_tray(ctx: egui::Context, engine: Engine) -> Option<tray_icon::TrayIcon> {
    #[cfg(target_os = "linux")]
    {
        // Spawn a dedicated thread for GTK tray icon initialization and event loop
        let engine_clone = engine.clone();
        std::thread::spawn({
            let ctx = ctx.clone();
            move || {
                gtk::glib::log_set_default_handler(|_, _, _| {});
                gtk::init().expect("Failed to initialize GTK for tray icon");

                use tray_icon::menu::{Menu, MenuItemBuilder, PredefinedMenuItem};
                let tray_menu = Menu::new();
                let open_item = MenuItemBuilder::new()
                    .text("Open Mouser-RS")
                    .enabled(true)
                    .build();
                let quit_item = MenuItemBuilder::new().text("Quit").enabled(true).build();
                tray_menu.append(&open_item).unwrap();
                tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
                tray_menu.append(&quit_item).unwrap();

                let open_id = open_item.id().clone();
                let quit_id = quit_item.id().clone();

                let _tray = tray_icon::TrayIconBuilder::new()
                    .with_menu(Box::new(tray_menu))
                    .with_tooltip("Mouser-rs")
                    .with_icon(gui::theme::create_mouse_tray_icon())
                    .build()
                    .expect("Failed to build tray icon");

                // Spawn menu event listener thread
                let engine_q = engine_clone.clone();
                std::thread::spawn(move || {
                    let menu_channel = tray_icon::menu::MenuEvent::receiver();
                    loop {
                        if let Ok(event) = menu_channel.recv() {
                            if event.id == open_id {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                                if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
                                    gui::theme::center_window(&ctx, monitor);
                                }
                            } else if event.id == quit_id {
                                engine_q.stop();
                                std::process::exit(0);
                            }
                        }
                    }
                });

                // Run GTK main loop to keep the tray icon alive and responsive
                gtk::main();
            }
        });
        None
    }

    #[cfg(not(target_os = "linux"))]
    {
        use tray_icon::menu::{Menu, MenuItemBuilder, PredefinedMenuItem};
        let tray_menu = Menu::new();
        let open_item = MenuItemBuilder::new()
            .text("Open Application (Mouser-rs)")
            .enabled(true)
            .build();
        let quit_item = MenuItemBuilder::new().text("Quit").enabled(true).build();
        tray_menu.append(&open_item).unwrap();
        tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
        tray_menu.append(&quit_item).unwrap();

        let open_id = open_item.id().clone();
        let quit_id = quit_item.id().clone();

        let icon = tray_icon::TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Mouser-rs")
            .with_icon(gui::theme::create_mouse_tray_icon())
            .build()
            .unwrap();

        let engine_q = engine.clone();
        std::thread::spawn({
            let ctx = ctx.clone();
            move || {
                let menu_channel = tray_icon::menu::MenuEvent::receiver();
                loop {
                    if let Ok(event) = menu_channel.recv() {
                        if event.id == open_id {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                            if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
                                gui::theme::center_window(&ctx, monitor);
                            }
                        } else if event.id == quit_id {
                            engine_q.stop();
                            std::process::exit(0);
                        }
                    }
                }
            }
        });
        Some(icon)
    }
}
