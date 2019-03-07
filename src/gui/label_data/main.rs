use imgui::*;
use ssl_log_tools::gui::{support, widgets};
use std::fs::File;
use std::io::prelude::*;

// type alias for multiple traits
trait SeekReadSend: Seek + Read + Send {}
impl<T: Seek + Read + Send> SeekReadSend for T {}

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

// icon font codes
const FA_SAVE: &str = "\u{f0c7}";

struct State {
    file_menu: FileMenu,
    player_widget: Option<widgets::LabelDataPlayer>,
}

impl State {
    pub fn new() -> State {
        State {
            file_menu: Default::default(),
            player_widget: None,
        }
    }
}

fn main() {
    let mut state = State::new();

    support::run("SSL Data Labeler".to_owned(), CLEAR_COLOR, |ui| {
        main_window(ui, &mut state)
    });
}

fn main_window<'a>(ui: &Ui<'a>, state: &mut State) -> bool {
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
            ui.menu_bar(|| {
                ui.menu(im_str!("File")).build(|| {
                    if ui
                        .menu_item(im_str!("Open"))
                        .shortcut(im_str!("Ctrl+o"))
                        .build()
                    {
                        state.file_menu.show_open_modal = true;
                    }
                    ui.menu_item(im_str!("{} Save", FA_SAVE))
                        .shortcut(im_str!("Ctrl+s"))
                        .enabled(false)
                        .build();
                    if ui.menu_item(im_str!("Save As..")).enabled(false).build() {
                        state.file_menu.show_save_as_modal = true;
                    }
                    ui.separator();
                    if ui.menu_item(im_str!("Exit")).build() {
                        state.file_menu.should_exit = true;
                    }
                });
            });

            match state.player_widget {
                Some(ref mut player_widget) => player_widget.build(ui),
                None => {}
            };

            if state.file_menu.show_open_modal {
                ui.open_popup(im_str!("Open Labeler Data File"));
                state.file_menu.show_open_modal = false;
            }
            ui.popup_modal(im_str!("Open Labeler Data File")).build(|| {
                match state.file_menu.open_file_browser.build(ui) {
                    Some(response) => match response {
                        widgets::FileDialogResponse::Select => {
                            let path = state
                                .file_menu
                                .open_file_browser
                                .current_selection()
                                .unwrap();
                            if path.is_dir() {
                                state.file_menu.open_file_browser.change_curr_dir(&path);
                            } else {
                                // open the player on the selected file
                                let reader: Box<SeekReadSend> =
                                    Box::new(File::open(&path).unwrap());

                                state.player_widget = Some(widgets::LabelDataPlayer::new(reader));

                                ui.close_current_popup();
                            }
                        }
                        _ => {
                            ui.close_current_popup();
                        }
                    },
                    None => {}
                };
            });
            if state.file_menu.show_save_as_modal {
                ui.open_popup(im_str!("Save Labels"));
            }
            ui.popup_modal(im_str!("Save Labels")).build(|| {
                ui.text("Save As modal");
                if ui.button(im_str!("OK"), (0.0, 0.0)) {
                    ui.close_current_popup();
                }
            });
        });

    !state.file_menu.should_exit
}

struct FileMenu {
    // open
    show_open_modal: bool,
    open_file_browser: widgets::FileBrowser,
    // save
    show_save_as_modal: bool,
    // exit
    should_exit: bool,
}

impl Default for FileMenu {
    fn default() -> Self {
        let filter_lists = vec![
            widgets::FileBrowserFilter::new("labeler", ".*\\.labeler").unwrap(),
            widgets::FileBrowserFilter::new("all", ".*").unwrap(),
        ];

        FileMenu {
            // open
            show_open_modal: false,
            open_file_browser: widgets::FileBrowser::new(None, Some(filter_lists)).unwrap(),
            // save
            show_save_as_modal: false,
            // exit
            should_exit: false,
        }
    }
}
