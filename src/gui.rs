use eframe::egui;

pub fn confirm_window(msg: String) {
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(eframe::egui::Vec2::new(300.0, 200.0));
    eframe::run_native(
        "sshield",
        native_options,
        Box::new(|cc| Box::new(ConfirmWindowApp::new(cc, msg))),
    )
    .unwrap();
}

#[derive(Default)]
struct ConfirmWindowApp {
    msg: String,
}

impl ConfirmWindowApp {
    fn new(_cc: &eframe::CreationContext<'_>, msg: String) -> Self {
        Self { msg }
    }
}

impl eframe::App for ConfirmWindowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("{self.msg}");
            if ui.button("Yes").clicked() {
                return;
            }
            if ui.button("No").clicked() {
                return;
            }
        });
    }
}
