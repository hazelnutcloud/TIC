#[tokio::main]
async fn main() -> eframe::Result<()> {
    eframe::run_native(
        "TIC",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(tic::TicApp::new(cc))),
    )
}
