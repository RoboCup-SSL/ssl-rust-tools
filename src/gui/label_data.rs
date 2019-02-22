use imgui::*;
use ssl_log_tools::labeler::player::Player as LabelerPlayer;
use std::io::prelude::*;

mod support;

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

trait SeekableReader: Read + Seek {}
impl<T: Read + Seek> SeekableReader for T {}

type BoxedSeekableReader = Box<SeekableReader>;

fn main() {
    let mut player_widget: PlayerWidget<BoxedSeekableReader> = PlayerWidget::new(None);

    support::run("SSL Data Labeler".to_owned(), CLEAR_COLOR, |ui| {
        main_window(ui, &mut player_widget)
    });
}

fn main_window<'a, T: Read + Seek>(ui: &Ui<'a>, player_widget: &mut PlayerWidget<T>) -> bool {
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
                    ui.menu_item(im_str!("Open"))
                        .shortcut(im_str!("Ctrl+o"))
                        .build();
                    ui.menu_item(im_str!("Save"))
                        .shortcut(im_str!("Ctrl+s"))
                        .enabled(false)
                        .build();
                    ui.menu_item(im_str!("Save As..")).enabled(false).build();
                    ui.separator();
                    ui.menu_item(im_str!("Exit")).build();
                });
            });
            player_widget.build(ui);
        });

    true
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
