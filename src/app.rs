#[derive(Default)]
pub struct TicApp {}

impl TicApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Default::default()
    }
}

impl eframe::App for TicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, world!");
        });
    }
}
