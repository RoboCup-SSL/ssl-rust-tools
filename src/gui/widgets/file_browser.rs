use imgui::*;
use regex::Regex;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{cmp, ffi, fmt, fs, io};

enum Selection {
    None,
    Directory(usize),
    File(usize),
}

pub enum DialogResponse {
    Select,
    Cancel,
}

pub struct FileFilter {
    display_name: String,
    regex: Regex,
}

impl FileFilter {
    pub fn new<S1: Into<String>, S2: AsRef<str>>(
        name: S1,
        regex: S2,
    ) -> Result<FileFilter, Box<dyn Error>> {
        let regex = Regex::new(regex.as_ref())?;

        Ok(FileFilter {
            display_name: name.into(),
            regex,
        })
    }

    pub fn is_match<S: AsRef<str>>(&self, text: S) -> bool {
        self.regex.is_match(text.as_ref())
    }
}

impl fmt::Display for FileFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "{}: ({:?})", self.display_name, self.regex)
    }
}

pub struct FileBrowser {
    curr_dir: PathBuf,
    filter_list: Vec<FileFilter>,
    curr_filter: i32,
    curr_selection: Selection,
    show_hidden: bool,

    // cached info for build method
    directories: Vec<PathBuf>,
    files: Vec<PathBuf>,
    filter_labels: Vec<ImString>,

    // drawing variables
    is_dirty: bool,
}

impl FileBrowser {
    pub fn new(
        curr_dir: Option<&Path>,
        filter_list: Option<Vec<FileFilter>>,
    ) -> Result<FileBrowser, io::Error> {
        let curr_dir = match curr_dir {
            Some(curr_dir) => curr_dir.to_path_buf(),
            None => PathBuf::from("."),
        };
        let curr_dir = curr_dir.canonicalize()?;

        let filter_list = match filter_list {
            Some(filter_list) => filter_list,
            None => {
                vec![FileFilter::new("all", ".*").expect("Failed to create default file filter")]
            }
        };

        let mut filter_labels = Vec::with_capacity(filter_list.len());
        for filter in &filter_list {
            filter_labels.push(ImString::new(format!("{}", filter)));
        }

        Ok(FileBrowser {
            // internal state
            curr_dir,
            filter_list,
            curr_filter: 0,
            curr_selection: Selection::None,
            show_hidden: false,

            // cached file stats
            directories: vec![],
            files: vec![],
            filter_labels,

            is_dirty: true,
        })
    }

    pub fn current_selection(&self) -> Option<PathBuf> {
        match self.curr_selection {
            Selection::Directory(ref index) => match *index {
                0 => Some(self.curr_dir.clone()),
                1 => {
                    let mut path = self.curr_dir.clone();
                    path.push("..");
                    Some(path.canonicalize().unwrap())
                }
                _ => Some(self.directories[*index].clone()),
            },
            Selection::File(ref index) => Some(self.files[*index].clone()),
            _ => None,
        }
    }

    pub fn change_curr_dir<P: AsRef<Path>>(&mut self, curr_dir: P) {
        self.curr_dir = curr_dir.as_ref().to_path_buf();
        self.is_dirty = true;
    }

    pub fn build<'ui>(&mut self, ui: &Ui<'ui>) -> Option<DialogResponse> {
        if self.is_dirty {
            self.curr_selection = Selection::None;

            self.directories.clear();
            self.directories.push(PathBuf::from("./"));
            self.directories.push(PathBuf::from("../"));

            self.files.clear();

            // walk current directory
            for entry in fs::read_dir(&self.curr_dir).expect("Failed to read directory") {
                let entry = entry.expect("Failed to read entry");
                let path = entry.path();

                // TODO(dschwab): Filter based on filter list
                if path.is_dir() {
                    self.directories.push(path);
                } else {
                    self.files.push(path);
                }
            }

            self.directories.sort_by(|a, b| {
                let a_str = a.to_string_lossy();
                let b_str = b.to_string_lossy();

                if a_str != b_str {
                    if a_str == "./" {
                        cmp::Ordering::Less
                    } else if a_str == "../" {
                        if b_str == "./" {
                            cmp::Ordering::Greater
                        } else {
                            cmp::Ordering::Less
                        }
                    } else {
                        a_str.cmp(&b_str)
                    }
                } else {
                    cmp::Ordering::Equal
                }
            });
            self.files.sort_unstable();

            self.is_dirty = false;
        }

        // layout stuff
        let parent_frame_size = {
            let frame_size = ui.frame_size();
            (
                frame_size.logical_size.0 as f32,
                frame_size.logical_size.1 as f32,
            )
        };

        let padding = ui.imgui().style().item_spacing;
        let text_height = ui
            .calc_text_size(im_str!("f"), false, parent_frame_size.0 / 2.0)
            .y;

        let line_height = text_height + 6.0 * padding.y;

        let child_frame_size = (
            parent_frame_size.0 - 2.0 * padding.x,
            // current directory text and buttons = 2 *
            // text_height
            parent_frame_size.1 - 4.0 * line_height,
        );

        let column_width = child_frame_size.1 / 2.0;
        let column_size = (column_width, text_height);

        ui.text(im_str!("{}", self.curr_dir.to_str().unwrap_or("ERROR")));

        let mut double_clicked = false;
        ui.child_frame(im_str!("Frame name"), child_frame_size)
            .show_borders(true)
            .build(|| {
                let curr_filter = self.filter_list.get(self.curr_filter as usize).unwrap();

                &ui.columns(2, im_str!(""), false);
                for (i, directory) in self.directories.iter().enumerate() {
                    let selected = match self.curr_selection {
                        Selection::Directory(ref index) => {
                            if *index == i {
                                true
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };

                    // don't try and extract file_name for default "." and ".." directories
                    let leaf = if i > 1 {
                        let leaf = directory
                            .file_name()
                            .and_then(ffi::OsStr::to_str)
                            .unwrap_or("ERROR");

                        // skip this node if it starts with a '.'
                        if leaf.starts_with(".") && !self.show_hidden {
                            continue;
                        }

                        format!("{}/", leaf)
                    } else {
                        directory.to_str().unwrap_or("ERROR").to_owned()
                    };

                    if ui.selectable(
                        im_str!("{}", leaf),
                        selected,
                        ImGuiSelectableFlags::AllowDoubleClick,
                        column_size,
                    ) {
                        self.curr_selection = Selection::Directory(i);
                        if ui.imgui().is_mouse_double_clicked(ImMouseButton::Left) {
                            double_clicked = true;
                            return;
                        }
                    }
                    ui.next_column();
                }

                for (i, file) in self.files.iter().enumerate() {
                    let selected = match self.curr_selection {
                        Selection::File(ref index) => {
                            if *index == i {
                                true
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };

                    let leaf = file
                        .file_name()
                        .and_then(ffi::OsStr::to_str)
                        .map(str::to_owned)
                        .unwrap_or_else(|| String::from("ERROR"));

                    // skip hidden files
                    if leaf.starts_with(".") && !self.show_hidden {
                        continue;
                    }

                    // does not match current filter list, so skip it
                    if !curr_filter.is_match(&leaf) {
                        continue;
                    }

                    if ui.selectable(
                        im_str!("{}", leaf),
                        selected,
                        ImGuiSelectableFlags::AllowDoubleClick,
                        column_size,
                    ) {
                        self.curr_selection = Selection::File(i);
                        if ui.imgui().is_mouse_double_clicked(ImMouseButton::Left) {
                            double_clicked = true;
                            return;
                        }
                    }
                    ui.next_column();
                }
            });

        ui.checkbox(im_str!("Show hidden"), &mut self.show_hidden);

        let filter_labels: Vec<&ImStr> = self.filter_labels.iter().map(ImString::as_ref).collect();
        ui.combo(
            im_str!("File Filter"),
            &mut self.curr_filter,
            &filter_labels,
            cmp::min(self.filter_list.len() as i32, 3),
        );

        let select_text = im_str!("Select");
        let select_text_size = ui.calc_text_size(select_text, false, parent_frame_size.0);

        if double_clicked || ui.button(im_str!("Select"), (0.0, 0.0)) {
            match self.curr_selection {
                Selection::None => {}
                _ => return Some(DialogResponse::Select),
            }
        }
        // TODO(dschwab): How to get button padding? need to replace
        // the 20.0 with actual dynamic padding size
        ui.same_line(select_text_size.x + 20.0);
        if ui.button(im_str!("Cancel"), (0.0, 0.0)) {
            return Some(DialogResponse::Cancel);
        }

        None
    }
}
