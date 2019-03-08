use clap::{App, Arg};
use imgui::*;
use ssl_rust_tools::gui::{support, widgets};
use std::fs;
use std::io::{BufReader, Seek, Read};
use std::path::Path;

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

// type alias for multiple traits
trait SeekReadSend: Seek + Read + Send {}
impl<T: Seek + Read + Send> SeekReadSend for T {}

struct State {
    player: widgets::LabelDataPlayer,
}

fn main() {
    let matches = App::new("Label Data Player Demo")
        .version("1.0")
        .author("Devin Schwab <dschwab@andrew.cmu.edu")
        .about("Label Data Player Widget Demo")
        .arg(
            Arg::with_name("LABEL_DATA_FILE")
                .help("Specifies label data file that should be played")
                .required(true)
                .index(1),
        )
        .get_matches();

    let label_data_path = Path::new(matches.value_of("LABEL_DATA_FILE").unwrap());
    let reader: Box<SeekReadSend> = Box::new(BufReader::new(fs::File::open(label_data_path).unwrap()));

    let mut state = State {
        player: widgets::LabelDataPlayer::new(reader),
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
            state.player.build(ui);
        });

    true
}
