use imgui::*;
use ssl_log_tools::gui::{support, widgets};
use std::path::PathBuf;

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

struct State {
    selected_file: Option<PathBuf>,
    file_browser: widgets::FileBrowser,
}

fn main() {
    let mut state = State {
        selected_file: None,
        file_browser: widgets::FileBrowser::new(
            None,
            Some(vec![
                widgets::FileBrowserFilter::new("all", ".*").unwrap(),
                widgets::FileBrowserFilter::new("txt", ".*\\.txt").unwrap(),
                widgets::FileBrowserFilter::new("sh", ".*\\.sh").unwrap(),
            ]),
        )
        .unwrap(),
    };

    support::run("File Browser Demo".to_owned(), CLEAR_COLOR, |ui| {
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
            match state.selected_file {
                Some(ref file_name) => ui.text(im_str!("Selected {:?}", file_name)),
                None => ui.text(im_str!("No file selected!")),
            };
            if ui.button(im_str!("Select file"), (0.0, 0.0)) {
                ui.open_popup(im_str!("Select file"));
            }
            ui.popup_modal(im_str!("Select file")).build(|| {
                match state.file_browser.build(ui) {
                    Some(response) => match response {
                        widgets::FileDialogResponse::Select => {
                            let path = state.file_browser.current_selection().unwrap();
                            if path.is_dir() {
                                state.file_browser.change_curr_dir(&path);
                            } else {
                                state.selected_file = Some(path);
                                ui.close_current_popup();
                            }
                        }
                        _ => {
                            state.selected_file = None;
                            ui.close_current_popup();
                        }
                    },
                    None => {}
                };
            });
        });

    true
}
