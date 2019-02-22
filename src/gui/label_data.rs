use imgui::*;
use ssl_log_tools::labeler::player::Player as LabelerPlayer;
use ssl_log_tools::labeler::reader::LabelerDataReader;
use std::env;
use std::fs::File;
use std::io::prelude::*;

mod support;

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

// icon font codes
const FA_SAVE: &str = "\u{f0c7}";

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
                    ui.menu_item(im_str!("Exit")).build();
                });
            });
            state.player_widget.build(ui);

            if state.file_menu.show_open_modal {
                ui.open_popup(im_str!("Open Labeler Data File"));
                state.file_menu.show_open_modal = false;
            }
            ui.popup(im_str!("Open Labeler Data File"), || {
                ui.text(im_str!(
                    "Current Working Dir: {:?}",
                    env::current_dir().unwrap()
                ));
                ui.input_text(
                    im_str!("File Path"),
                    &mut state.file_menu.label_data_file_path,
                )
                .build();
                if ui.button(im_str!("Open"), (0.0, 0.0)) {
                    let mut file_path = env::current_dir().unwrap();
                    file_path.push(state.file_menu.label_data_file_path.to_str());
                    let file_path = file_path.canonicalize().unwrap();

                    let reader: BoxedSeekableReader = Box::new(File::open(&file_path).unwrap());
                    let reader = LabelerDataReader::new(reader).unwrap();
                    let player = LabelerPlayer::new(reader);
                    state.player_widget.set_player(player);

                    ui.close_current_popup();
                }
                if ui.button(im_str!("Cancel"), (0.0, 0.0)) {
                    ui.close_current_popup();
                }
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

    true
}

struct FileMenu {
    show_open_modal: bool,
    show_save_as_modal: bool,
    label_data_file_path: ImString,
}

impl Default for FileMenu {
    fn default() -> Self {
        FileMenu {
            show_open_modal: false,
            show_save_as_modal: false,
            label_data_file_path: ImString::with_capacity(1024),
        }
    }
}

struct PlayerWidget<T: SeekableReader> {
    player: Option<LabelerPlayer<T>>,
    frame_index: i32,
}

impl<T> PlayerWidget<T>
where
    T: SeekableReader,
{
    fn new(player: Option<LabelerPlayer<T>>) -> PlayerWidget<T> {
        PlayerWidget {
            player,
            frame_index: 0,
        }
    }

    fn get_frame(&self) -> Option<usize> {
        Some(self.frame_index as usize)
    }

    fn set_player(&mut self, player: LabelerPlayer<T>) -> &mut Self {
        self.player = Some(player);
        self.frame_index = 0;

        self
    }

    fn build<'a>(&mut self, ui: &Ui<'a>) -> bool {
        match &self.player {
            Some(player) => {
                if ui
                    .slider_int(
                        im_str!("Frame"),
                        &mut self.frame_index,
                        0,
                        player.len() as i32,
                    )
                    .build()
                {
                    println!("Value changed to: {}", self.frame_index);
                }
            }
            None => {
                // TODO(dschwab): Show disabled widget
                ui.text(im_str!("No labeler data file loaded!"));
            }
        }

        true
    }
}
