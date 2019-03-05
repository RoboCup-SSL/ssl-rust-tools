use imgui::*;
use ssl_log_tools::gui::{support, widgets};
use ssl_log_tools::labeler::player::Player as LabelerPlayer;
use ssl_log_tools::labeler::reader::LabelerDataReader;
use std::env;
use std::fs::File;
use std::io::prelude::*;

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

// icon font codes
const FA_SAVE: &str = "\u{f0c7}";
const FA_REWIND: &str = "\u{f04a}";
const FA_STEP_BACK: &str = "\u{f048}";
const FA_PAUSE: &str = "\u{f04c}";
const FA_STEP_FORWARD: &str = "\u{f051}";
const FA_FAST_FORWARD: &str = "\u{f04e}";

trait SeekableReader: Read + Seek {}
impl<T: Read + Seek> SeekableReader for T {}

type BoxedSeekableReader = Box<SeekableReader>;

struct State {
    file_menu: FileMenu,
    player_widget: PlayerWidget<BoxedSeekableReader>,
}

impl State {
    pub fn new() -> State {
        State {
            file_menu: Default::default(),
            player_widget: PlayerWidget::new(None),
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
            state.player_widget.build(ui);

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
                                let reader: BoxedSeekableReader =
                                    Box::new(File::open(&path).unwrap());
                                let reader = LabelerDataReader::new(reader).unwrap();
                                let player = LabelerPlayer::new(reader);
                                state.player_widget.set_player(player);

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

enum PlayerWidgetState {
    Paused,
    Forward,
    Backward,
}

struct PlayerWidget<T: SeekableReader> {
    player: Option<LabelerPlayer<T>>,
    frame_index: i32,
    playback_speed: f32,
    widget_state: PlayerWidgetState,
}

impl<T> PlayerWidget<T>
where
    T: SeekableReader,
{
    fn new(player: Option<LabelerPlayer<T>>) -> PlayerWidget<T> {
        PlayerWidget {
            player,
            frame_index: 0,
            playback_speed: 1.0,
            widget_state: PlayerWidgetState::Paused,
        }
    }

    fn get_frame(&self) -> Option<usize> {
        Some(self.frame_index as usize)
    }

    fn set_player(&mut self, player: LabelerPlayer<T>) -> &mut Self {
        // set the first frame playing
        player.play_frame(0);

        self.player = Some(player);
        self.frame_index = 0;

        self
    }

    fn build<'a>(&mut self, ui: &Ui<'a>) -> bool {
        match &self.player {
            Some(player) => {
                ui.input_float(im_str!("Playback Speed"), &mut self.playback_speed)
                    .build();
                ui.slider_float(im_str!(""), &mut self.playback_speed, 0.0, 10.0)
                    .build();

                if ui
                    .slider_int(
                        im_str!("Frame"),
                        &mut self.frame_index,
                        0,
                        player.len() as i32,
                    )
                    .build()
                {
                    player.play_frame(self.frame_index as usize);
                }

                ui.button(im_str!("{}", FA_REWIND), (0.0, 0.0));
                ui.same_line(100.0);
                ui.button(im_str!("{}", FA_STEP_BACK), (0.0, 0.0));
                ui.same_line(200.0);
                ui.button(im_str!("{}", FA_PAUSE), (0.0, 0.0));
                ui.same_line(300.0);
                ui.button(im_str!("{}", FA_STEP_FORWARD), (0.0, 0.0));
                ui.same_line(400.0);
                ui.button(im_str!("{}", FA_FAST_FORWARD), (0.0, 0.0));
            }
            None => {
                // TODO(dschwab): Show disabled widget
                ui.text(im_str!("No labeler data file loaded!"));
            }
        }

        true
    }
}