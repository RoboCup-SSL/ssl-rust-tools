use imgui::*;
use ssl_rust_tools::gui::{support, widgets};

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

struct State {
    tabs: widgets::TabBar,
}

fn main() {
    let mut state = State {
        tabs: widgets::TabBar::new(vec![
            String::from("Tab 1"),
            String::from("Also a tab"),
            String::from("Last tab!"),
        ]),
    };

    support::run("Label Data Player Demo".to_owned(), CLEAR_COLOR, |ui| {
        main_window(ui, &mut state)
    });
}

fn main_window<'ui>(ui: &Ui<'ui>, state: &mut State) -> bool {
    let window_size = {
        let frame_size = ui.frame_size();
        (
            frame_size.logical_size.0 as f32,
            frame_size.logical_size.1 as f32,
        )
    };

    ui.window(im_str!("Label Data Player Demo"))
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .scrollable(true)
        .size(window_size, ImGuiCond::Always)
        .collapsible(false)
        .menu_bar(true)
        .position((0.0, 0.0), ImGuiCond::Always)
        .build(|| {
            match state.tabs.build(ui) {
                0 => ui.text("Tab 1 selected"),
                1 => ui.text("Tab 2 selected"),
                2 => ui.text("Tab 3 selected"),
                _ => panic!("Unhandled tab"),
            };

        });

    true
}
