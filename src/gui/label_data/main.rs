use imgui::*;
use protobuf::ProtobufEnum;
use ssl_log_tools::gui::{support, widgets};
use ssl_log_tools::protos;
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
    tabs: widgets::TabBar,
    dribbling_labels: Vec<protos::log_labels::DribblingLabel>,
    ball_possession_labels: Vec<protos::log_labels::BallPossessionLabel>,
}

impl State {
    pub fn new() -> State {
        State {
            file_menu: Default::default(),
            player_widget: None,
            tabs: widgets::TabBar::new(vec![
                "Dribbling",
                "Ball Possession",
                "Passing",
                "Goal Shots",
            ]),
            dribbling_labels: vec![],
            ball_possession_labels: vec![],
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
                    if ui.menu_item(im_str!("Open data file")).build() {
                        state.file_menu.show_open_data_file_modal = true;
                    }
                    if ui
                        .menu_item(im_str!("Open label file"))
                        .enabled(false)
                        .build()
                    {
                        state.file_menu.show_open_label_file_modal = true;
                    }
                    ui.menu_item(im_str!("{} Save", FA_SAVE))
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
                Some(ref mut player_widget) => {
                    player_widget.build(ui);
                    match state.tabs.build(ui) {
                        0 => dribbling_tab(ui, state),
                        1 => ball_possession_tab(ui, state),
                        2 => passing_tab(ui, state),
                        3 => goal_shot_tab(ui, state),
                        _ => panic!("Unhandled tab"),
                    }
                }
                None => {
                    ui.text("Open a labeler data file to begin");
                }
            };

            if state.file_menu.show_open_data_file_modal {
                ui.open_popup(im_str!("Open Labeler Data File"));
                state.file_menu.show_open_data_file_modal = false;
            }
            ui.popup_modal(im_str!("Open Labeler Data File")).build(|| {
                match state.file_menu.open_data_file_browser.build(ui) {
                    Some(response) => match response {
                        widgets::FileDialogResponse::Select => {
                            let path = state
                                .file_menu
                                .open_data_file_browser
                                .current_selection()
                                .unwrap();
                            if path.is_dir() {
                                state
                                    .file_menu
                                    .open_data_file_browser
                                    .change_curr_dir(&path);
                            } else {
                                // open the player on the selected file
                                let reader: Box<SeekReadSend> =
                                    Box::new(File::open(&path).unwrap());

                                let player_widget = widgets::LabelDataPlayer::new(reader);
                                // allocate the label messages
                                state.dribbling_labels = Vec::with_capacity(player_widget.len());
                                for _ in 0..player_widget.len() {
                                    state
                                        .dribbling_labels
                                        .push(protos::log_labels::DribblingLabel::new());
                                }
                                state.ball_possession_labels =
                                    Vec::with_capacity(player_widget.len());
                                for _ in 0..player_widget.len() {
                                    state
                                        .ball_possession_labels
                                        .push(protos::log_labels::BallPossessionLabel::new());
                                }

                                state.player_widget = Some(player_widget);

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
            if state.file_menu.show_open_label_file_modal {
                ui.open_popup(im_str!("Open Label File"));
                state.file_menu.show_open_label_file_modal = false;
            }
            ui.popup_modal(im_str!("Open Label File")).build(|| {
                match state.file_menu.open_label_file_browser.build(ui) {
                    Some(response) => match response {
                        widgets::FileDialogResponse::Select => {
                            let path = state
                                .file_menu
                                .open_label_file_browser
                                .current_selection()
                                .unwrap();
                            if path.is_dir() {
                                state
                                    .file_menu
                                    .open_label_file_browser
                                    .change_curr_dir(&path);
                            } else {
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
    // open data file
    show_open_data_file_modal: bool,
    open_data_file_browser: widgets::FileBrowser,
    // open label file
    show_open_label_file_modal: bool,
    open_label_file_browser: widgets::FileBrowser,
    // save
    show_save_as_modal: bool,
    // exit
    should_exit: bool,
}

impl Default for FileMenu {
    fn default() -> Self {
        let data_file_filter_lists = vec![
            widgets::FileBrowserFilter::new("labeler", ".*\\.labeler").unwrap(),
            widgets::FileBrowserFilter::new("all", ".*").unwrap(),
        ];
        let label_file_filter_lists = vec![
            widgets::FileBrowserFilter::new("label", ".*\\.label").unwrap(),
            widgets::FileBrowserFilter::new("all", ".*").unwrap(),
        ];

        FileMenu {
            // open data file
            show_open_data_file_modal: false,
            open_data_file_browser: widgets::FileBrowser::new(None, Some(data_file_filter_lists))
                .unwrap(),
            // open label file
            show_open_label_file_modal: false,
            open_label_file_browser: widgets::FileBrowser::new(None, Some(label_file_filter_lists))
                .unwrap(),
            // save
            show_save_as_modal: false,
            // exit
            should_exit: false,
        }
    }
}

fn dribbling_tab<'a>(ui: &Ui<'a>, state: &mut State) {
    let player_widget = state.player_widget.as_ref().unwrap();
    let curr_frame = player_widget.curr_frame();

    let dribbling_label = &mut state.dribbling_labels[curr_frame];

    let mut is_dribbling = dribbling_label.get_is_dribbling();
    if ui.checkbox(im_str!("Is Dribbling?"), &mut is_dribbling) {
        dribbling_label.set_is_dribbling(is_dribbling);
    }

    if is_dribbling {
        let mut robot_id = dribbling_label.get_robot_id() as i32;
        if ui.input_int(im_str!("Robot ID"), &mut robot_id).build() {
            dribbling_label.set_robot_id(robot_id as u32);
        }

        let item_strings = vec![ImString::new("Yellow"), ImString::new("Blue")];
        let item_strs: Vec<&ImStr> = item_strings.iter().map(ImString::as_ref).collect();
        let mut team = dribbling_label.get_team().value();
        if ui.combo(im_str!("Team"), &mut team, &item_strs, 2) {
            let team = match protos::log_labels::Team::from_i32(team) {
                Some(team) => team,
                None => {
                    eprintln!("Invalid team id: {}", team);
                    dribbling_label.get_team()
                }
            };
            dribbling_label.set_team(team);
        }
    }
}

fn ball_possession_tab<'a>(ui: &Ui<'a>, state: &mut State) {
    let player_widget = state.player_widget.as_ref().unwrap();
    let curr_frame = player_widget.curr_frame();

    let ball_possession_label = &mut state.ball_possession_labels[curr_frame];

    let item_strings = vec![
        ImString::new("None"),
        ImString::new("Blue"),
        ImString::new("Yellow"),
    ];
    let item_strs: Vec<&ImStr> = item_strings.iter().map(ImString::as_ref).collect();
    let mut ball_possession_state = ball_possession_label.get_state().value();
    if ui.combo(
        im_str!("Who Possesses the ball?"),
        &mut ball_possession_state,
        &item_strs,
        3,
    ) {
        let ball_possession_state =
            match protos::log_labels::BallPossessionLabel_State::from_i32(ball_possession_state) {
                Some(ball_possession_state) => ball_possession_state,
                None => {
                    eprintln!("Invalid ball possession state: {}", ball_possession_state);
                    ball_possession_label.get_state()
                }
            };
        ball_possession_label.set_state(ball_possession_state);
    }

    if ball_possession_state != protos::log_labels::BallPossessionLabel_State::NONE.value() {
        let mut robot_id = ball_possession_label.get_robot_id() as i32;
        if ui.input_int(im_str!("Robot ID"), &mut robot_id).build() {
            ball_possession_label.set_robot_id(robot_id as u32);
        }
    }
}

fn passing_tab<'a>(ui: &Ui<'a>, state: &mut State) {
    ui.text("passing tab");
}

fn goal_shot_tab<'a>(ui: &Ui<'a>, state: &mut State) {
    ui.text("goal shot tab");
}
