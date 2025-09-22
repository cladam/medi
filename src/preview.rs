use eframe::{egui, App, Frame};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

// This struct holds the state of our preview app.
pub struct PreviewApp {
    content: String,
    cache: CommonMarkCache,
}

impl PreviewApp {
    // Create a new instance of the app with the note's content.
    pub fn new(content: String) -> Self {
        Self {
            content,
            cache: CommonMarkCache::default(),
        }
    }
}

// This is the core of the egui app. The `update` function is called on every frame.
impl App for PreviewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Use the CommonMarkViewer to render the Markdown.
                CommonMarkViewer::new().show(ui, &mut self.cache, &self.content);
            });
        });
    }
}
