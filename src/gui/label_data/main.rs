use imgui::*;
use protobuf::{Message, ProtobufEnum, RepeatedField};
use ssl_rust_tools::gui::{support, widgets};
use ssl_rust_tools::protos;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

// type alias for multiple traits
trait SeekReadSend: Seek + Read + Send {}
impl<T: Seek + Read + Send> SeekReadSend for T {}

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

// icon font codes
const FA_SAVE: &str = "\u{f0c7}";
const FA_PLUS: &str = "\u{f0fe}";
const FA_TRASH: &str = "\u{f1f8}";

struct State {
    file_menu: FileMenu,
    player_widget: Option<widgets::LabelDataPlayer>,
    tabs: widgets::TabBar,
    dribbling_labels: Vec<protos::log_labels::DribblingLabel>,
    ball_possession_labels: Vec<protos::log_labels::BallPossessionLabel>,
    curr_passing_label: usize,
    passing_labels: Vec<protos::log_labels::PassingLabel>,
    curr_goal_shot_label: usize,
    goal_shot_labels: Vec<protos::log_labels::GoalShotLabel>,
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
            curr_passing_label: 0,
            passing_labels: vec![],
            curr_goal_shot_label: 0,
            goal_shot_labels: vec![],
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
            let data_file_loaded = state.player_widget.is_some();

            ui.menu_bar(|| {
                ui.menu(im_str!("File")).build(|| {
                    if ui.menu_item(im_str!("Open data file")).build() {
                        state.file_menu.show_open_data_file_modal = true;
                    }
                    if ui
                        .menu_item(im_str!("Open label file"))
                        .enabled(data_file_loaded)
                        .build()
                    {
                        state.file_menu.show_open_label_file_modal = true;
                    }
                    if ui
                        .menu_item(im_str!("{} Save", FA_SAVE))
                        .enabled(data_file_loaded)
                        .build()
                    {
                        save_labels(state);
                    }
                    if ui
                        .menu_item(im_str!("Save As.."))
                        .enabled(data_file_loaded)
                        .build()
                    {
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
                                load_labels(state);
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
                state.file_menu.show_save_as_modal = false;
            }
            ui.popup_modal(im_str!("Save Labels")).build(|| {
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
                                state.file_menu.save_path = Some(path);
                                save_labels(state);
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
    save_path: Option<PathBuf>,
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
            save_path: None,
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
    let parent_frame = {
        let frame_size = ui.frame_size().logical_size;
        (frame_size.0 as f32, frame_size.1 as f32)
    };
    let start_pos = ui.get_cursor_screen_pos();
    let child_frame_size = (0.333 * parent_frame.0, parent_frame.1 - 6.0 * start_pos.1);

    let add_label = im_str!("{} Add", FA_PLUS);
    if ui.button(add_label, (0.0, 0.0)) {
        let new_label = protos::log_labels::PassingLabel::new();
        state.passing_labels.push(new_label);

        state.curr_passing_label = state.passing_labels.len() - 1;
    }
    let text_size = ui.calc_text_size(add_label, false, parent_frame.0);
    ui.same_line(text_size.x + 20.0);
    if ui.button(im_str!("{} Delete", FA_TRASH), (0.0, 0.0)) {
        if state.curr_passing_label < state.passing_labels.len() {
            state.passing_labels.remove(state.curr_passing_label);
            if state.curr_passing_label != 0 {
                state.curr_passing_label -= 1;
            }
        }
    }

    let list_start_pos = ui.get_cursor_screen_pos();
    let mut curr_selection: Box<usize> = Box::new(state.curr_passing_label);
    ui.child_frame(im_str!("passing_select_list"), child_frame_size)
        .show_borders(true)
        .build(|| {
            for (i, passing_label) in state.passing_labels.iter().enumerate() {
                let selected = i == *curr_selection;
                if ui.selectable(
                    im_str!(
                        "{} -- {}",
                        passing_label.get_start_frame(),
                        passing_label.get_end_frame(),
                    ),
                    selected,
                    ImGuiSelectableFlags::AllowDoubleClick,
                    (child_frame_size.0, text_size.y),
                ) {
                    *curr_selection = i;
                }
            }
        });
    state.curr_passing_label = *curr_selection;

    let x_offset = list_start_pos.0 + child_frame_size.0 + 20.0;
    ui.set_cursor_screen_pos((x_offset, list_start_pos.1));

    ui.push_item_width(0.4 * parent_frame.0 - 40.0);

    if state.passing_labels.is_empty() {
        ui.pop_item_width();
        return;
    }

    let passing_label = &mut state.passing_labels[state.curr_passing_label];

    let mut start_frame = passing_label.get_start_frame() as i32;
    if ui
        .input_int(im_str!("Start Frame"), &mut start_frame)
        .build()
    {
        passing_label.set_start_frame(start_frame as u64);
        if start_frame > passing_label.get_end_frame() as i32 {
            passing_label.set_end_frame(start_frame as u64);
        }
    }
    ui.set_cursor_screen_pos((x_offset, ui.get_cursor_screen_pos().1));

    let mut end_frame = passing_label.get_end_frame() as i32;
    if ui.input_int(im_str!("End Frame"), &mut end_frame).build() {
        passing_label.set_end_frame(end_frame as u64);
        if end_frame < passing_label.get_start_frame() as i32 {
            passing_label.set_start_frame(end_frame as u64);
        }
    }
    ui.set_cursor_screen_pos((x_offset, ui.get_cursor_screen_pos().1));

    let mut successful = passing_label.get_successful();
    if ui.checkbox(im_str!("Successful?"), &mut successful) {
        passing_label.set_successful(successful);
    }
    ui.set_cursor_screen_pos((x_offset, ui.get_cursor_screen_pos().1));

    let mut passer_id = passing_label.get_passer_id() as i32;
    if ui.input_int(im_str!("Passer ID"), &mut passer_id).build() {
        passing_label.set_passer_id(passer_id as u32);
    }
    ui.set_cursor_screen_pos((x_offset, ui.get_cursor_screen_pos().1));

    let item_strings = vec![ImString::new("Yellow"), ImString::new("Blue")];
    let item_strs: Vec<&ImStr> = item_strings.iter().map(ImString::as_ref).collect();
    let mut team = passing_label.get_passer_team().value();
    if ui.combo(im_str!("Passer Team"), &mut team, &item_strs, 2) {
        let team = match protos::log_labels::Team::from_i32(team) {
            Some(team) => team,
            None => {
                eprintln!("Invalid team id: {}", team);
                passing_label.get_passer_team()
            }
        };
        passing_label.set_passer_team(team);
    }
    ui.set_cursor_screen_pos((x_offset, ui.get_cursor_screen_pos().1));

    let mut receiver_id = passing_label.get_receiver_id() as i32;
    if ui
        .input_int(im_str!("Receiver ID"), &mut receiver_id)
        .build()
    {
        passing_label.set_receiver_id(receiver_id as u32);
    }
    ui.set_cursor_screen_pos((x_offset, ui.get_cursor_screen_pos().1));

    ui.pop_item_width();
}

fn goal_shot_tab<'a>(ui: &Ui<'a>, state: &mut State) {
    let parent_frame = {
        let frame_size = ui.frame_size().logical_size;
        (frame_size.0 as f32, frame_size.1 as f32)
    };
    let start_pos = ui.get_cursor_screen_pos();
    let child_frame_size = (0.333 * parent_frame.0, parent_frame.1 - 6.0 * start_pos.1);

    let add_label = im_str!("{} Add", FA_PLUS);
    if ui.button(add_label, (0.0, 0.0)) {
        let new_label = protos::log_labels::GoalShotLabel::new();
        state.goal_shot_labels.push(new_label);

        state.curr_goal_shot_label = state.goal_shot_labels.len() - 1;
    }
    let text_size = ui.calc_text_size(add_label, false, parent_frame.0);
    ui.same_line(text_size.x + 20.0);
    if ui.button(im_str!("{} Delete", FA_TRASH), (0.0, 0.0)) {
        if state.curr_goal_shot_label < state.goal_shot_labels.len() {
            state.goal_shot_labels.remove(state.curr_goal_shot_label);
            if state.curr_goal_shot_label != 0 {
                state.curr_goal_shot_label -= 1;
            }
        }
    }

    let list_start_pos = ui.get_cursor_screen_pos();
    let mut curr_selection: Box<usize> = Box::new(state.curr_goal_shot_label);
    ui.child_frame(im_str!("goal_shot_select_list"), child_frame_size)
        .show_borders(true)
        .build(|| {
            for (i, goal_shot_label) in state.goal_shot_labels.iter().enumerate() {
                let selected = i == *curr_selection;
                if ui.selectable(
                    im_str!(
                        "{} -- {}",
                        goal_shot_label.get_start_frame(),
                        goal_shot_label.get_end_frame(),
                    ),
                    selected,
                    ImGuiSelectableFlags::AllowDoubleClick,
                    (child_frame_size.0, text_size.y),
                ) {
                    *curr_selection = i;
                }
            }
        });
    state.curr_goal_shot_label = *curr_selection;

    let x_offset = list_start_pos.0 + child_frame_size.0 + 20.0;
    ui.set_cursor_screen_pos((x_offset, list_start_pos.1));

    ui.push_item_width(0.4 * parent_frame.0 - 40.0);

    if state.goal_shot_labels.is_empty() {
        ui.pop_item_width();
        return;
    }

    let goal_shot_label = &mut state.goal_shot_labels[state.curr_goal_shot_label];

    let mut start_frame = goal_shot_label.get_start_frame() as i32;
    if ui
        .input_int(im_str!("Start Frame"), &mut start_frame)
        .build()
    {
        goal_shot_label.set_start_frame(start_frame as u64);
        if start_frame > goal_shot_label.get_end_frame() as i32 {
            goal_shot_label.set_end_frame(start_frame as u64);
        }
    }
    ui.set_cursor_screen_pos((x_offset, ui.get_cursor_screen_pos().1));

    let mut end_frame = goal_shot_label.get_end_frame() as i32;
    if ui.input_int(im_str!("End Frame"), &mut end_frame).build() {
        goal_shot_label.set_end_frame(end_frame as u64);
        if end_frame < goal_shot_label.get_start_frame() as i32 {
            goal_shot_label.set_start_frame(end_frame as u64);
        }
    }
    ui.set_cursor_screen_pos((x_offset, ui.get_cursor_screen_pos().1));

    let mut successful = goal_shot_label.get_successful();
    if ui.checkbox(im_str!("Successful?"), &mut successful) {
        goal_shot_label.set_successful(successful);
    }
    ui.set_cursor_screen_pos((x_offset, ui.get_cursor_screen_pos().1));

    let mut shooter_id = goal_shot_label.get_shooter_id() as i32;
    if ui.input_int(im_str!("Shooter ID"), &mut shooter_id).build() {
        goal_shot_label.set_shooter_id(shooter_id as u32);
    }
    ui.set_cursor_screen_pos((x_offset, ui.get_cursor_screen_pos().1));

    let item_strings = vec![ImString::new("Yellow"), ImString::new("Blue")];
    let item_strs: Vec<&ImStr> = item_strings.iter().map(ImString::as_ref).collect();
    let mut team = goal_shot_label.get_shooter_team().value();
    if ui.combo(im_str!("Shooter Team"), &mut team, &item_strs, 2) {
        let team = match protos::log_labels::Team::from_i32(team) {
            Some(team) => team,
            None => {
                eprintln!("Invalid team id: {}", team);
                goal_shot_label.get_shooter_team()
            }
        };
        goal_shot_label.set_shooter_team(team);
    }
    ui.set_cursor_screen_pos((x_offset, ui.get_cursor_screen_pos().1));

    ui.pop_item_width();
}

fn save_labels(state: &mut State) {
    // create the protobuf file
    let mut labels = protos::log_labels::Labels::new();
    labels.set_dribbling_labels(RepeatedField::from_vec(state.dribbling_labels.clone()));
    labels.set_ball_possession_labels(RepeatedField::from_vec(
        state.ball_possession_labels.clone(),
    ));

    // sort the passing labels by the start frame
    let mut passing_labels = state.passing_labels.clone();
    passing_labels.sort_by(|a, b| a.get_start_frame().cmp(&b.get_start_frame()));
    labels.set_passing_labels(RepeatedField::from(passing_labels));

    // sort goal shot labels by start frame
    let mut goal_shot_labels = state.goal_shot_labels.clone();
    goal_shot_labels.sort_by(|a, b| a.get_start_frame().cmp(&b.get_start_frame()));
    labels.set_goal_shot_labels(RepeatedField::from(state.goal_shot_labels.clone()));

    // serialize the protobuf
    let msg_bytes = labels.write_to_bytes().unwrap();

    // default file name is same as opened file but with the .label extension
    let label_file_path = match state.file_menu.save_path {
        Some(ref save_path) => save_path,
        None => {
            let mut label_file_path = state
                .file_menu
                .open_data_file_browser
                .current_selection()
                .unwrap();
            label_file_path.set_extension("label");

            state.file_menu.save_path = Some(label_file_path);

            state.file_menu.save_path.as_ref().unwrap()
        }
    };

    // open the file for writing, overwriting any data currently there
    let mut label_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(label_file_path)
        .unwrap();
    label_file.write_all(&msg_bytes).unwrap();
}

fn load_labels(state: &mut State) {
    let label_file_path = state
        .file_menu
        .open_label_file_browser
        .current_selection()
        .unwrap();

    let mut label_file = fs::File::open(label_file_path.clone()).unwrap();
    let labels: protos::log_labels::Labels = protobuf::parse_from_reader(&mut label_file).unwrap();

    let dribbling_labels = labels.get_dribbling_labels().to_vec();
    let ball_possession_labels = labels.get_ball_possession_labels().to_vec();

    // TODO(dschwab): Show these logs in the gui
    if dribbling_labels.len() != ball_possession_labels.len() {
        eprintln!(
            "Dribbling labels and ball possession labels do not match length. {} != {}",
            dribbling_labels.len(),
            ball_possession_labels.len()
        );
        return;
    }
    if dribbling_labels.len() != state.player_widget.as_ref().unwrap().len() {
        eprintln!(
            "Number of instantaneous labels does not match number of frames in data file. {} != {}",
            dribbling_labels.len(),
            state.player_widget.as_ref().unwrap().len()
        );
        return;
    }

    let passing_labels = labels.get_passing_labels().to_vec();
    let goal_shot_labels = labels.get_goal_shot_labels().to_vec();

    state.dribbling_labels = dribbling_labels;
    state.ball_possession_labels = ball_possession_labels;
    state.passing_labels = passing_labels;
    state.goal_shot_labels = goal_shot_labels;

    state.file_menu.save_path = Some(label_file_path);
}
