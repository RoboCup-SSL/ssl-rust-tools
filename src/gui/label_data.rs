use imgui::*;
mod support;

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

fn main() {
    support::run("SSL Data Labeler".to_owned(), CLEAR_COLOR, |ui, _, _| {
        main_window(ui)
    });
}

fn main_window<'a>(ui: &Ui<'a>) -> bool {
    let window_size = {
        let frame_size = ui.frame_size();
        (
            frame_size.logical_size.0 as f32,
            frame_size.logical_size.1 as f32,
        )
    };

    let styles = [StyleVar::WindowRounding(0.0)];
    ui.with_style_vars(&styles, || {
        ui.window(im_str!("SSL Data Labeler"))
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .scrollable(true)
            .size(window_size, ImGuiCond::Always)
            .collapsible(false)
            .menu_bar(true)
            .position((0.0, 0.0), ImGuiCond::Always)
            .build(|| {
                ui.text("Hello world!");
            });
    });

    true
}
