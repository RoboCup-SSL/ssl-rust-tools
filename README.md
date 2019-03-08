# SSL Rust Tools

This package contains a rust library for working with [SSL RoboCup log
files](https://github.com/RoboCup-SSL/ssl-logtools) as well as a
number of binaries for playing, filtering and labeling log file
events. See the following sections for more information on the
utilities included in this packages.

## Building

### Dependencies

This package is written in rust and should work with rust 1.33 or
later. To install rust visit
<https://www.rust-lang.org/tools/install>.

You will also need protobuf 3.6.1 or higher. You can install it via
the system package manager, or you can use the provided
`make_conan_venv.sh` script to create a locally installed version
using [Conan](https://conan.io/). 

If using the script, install conan and then run the script. It will
produce a folder called `venv`. To use the virtualenv run `source
venv/activate_run.sh`. This will set the appropriate environment
variables so that `protoc` and the protobuf libraries can be found
during the build. Note: After building, the virtualenv is not needed
to run as all libraries are statically linked.

### Building

In the root folder run: `cargo build --release`. This will build the
core library and all command line utilities.

If you want to build the labeling gui run `cargo build --release
--features=gui`.

## Command Line Utilities

### play_log

The `play_log` command will play an uncompressed log file so that it
can be visualized by tools such as [SSL Vision
Client](https://github.com/RoboCup-SSL/ssl-vision-client).

### make_labeler_data_file

This tool will pre-process a saved log file to prepare it for log
event labeling. The output labeler data file is the format used in the
[SSL RoboCup 2019 Technical
Challenge](https://github.com/RoboCup-SSL/technical-challenge-rules).

The log file must be uncompressed. If you have issues uncompressing a
log file due to `unexpected EOF` errors from gzip then you can use the
`zcat` utility to stream the extraction.

Either save the zcat output to a file:

``` shell
zcat compressed_log_file.log.gz > uncompressed_log_file.log
```

Or you can stream the zcat output directly to this tool using `-` to
represent stdin on the input file argument. Like so:

``` shell
zcat compressed_log_file.log.gz | make_labeler_data_file - output.labeler
```

Note: This filtering can take some time. If you use a saved file, the
progress indicators will be more useful, as the total file size can be
calculated up-front.

More details about the output file format can be found below

The filtering keeps only messages during ref stages that are running
(i.e. NORMAL_FIRST_HALF, NORMAL_SECOND_HALF, EXTRA_FIRST_HALF,
EXTRA_SECOND_HALF). As well as during ref commands that have actually
started the game play (i.e. NORMAL_START, FORCE_START,
DIRECT_FREE_YELLOW, DIRECT_FREE_BLUE, INDIRECT_FREE_YELLOW,
INDIRECT_FREE_BLUE).

When multiple cameras are running SSL Vision, the recorded messages
are asynchronously sent. Therefore there is no global clock step that
can be used for a frame. So instead frames are grouped together until
a camera repeats in the log. This results in most frame messages
containing a single message from each camera, however, in some cases
one or more of the camera messages will be missing.

Messages within a frame retain their original order according to their
order in the log file. The frame just introduces an artificial
timestep to the file so that labels can correspond to specific frames
rather than arbitrary timestamps.

### play_labeler_data_file

Play a labeler data file. Similar to playing a normal log, but this
contains only the filtered messages grouped into frames.

### score_label_file

Will output each label category score given a ground truth and a
predicted label. Will be used to score the technical challenge during
the competition.

## Label Data GUI

This package also contains a GUI tool for playing and labeling labeler
data files. A screenshot is shown below.

![Label Data GUI Screenshot](/screenshots/label_data_gui.png?raw=true
"Label Data GUI")

To start you must open a data file using the file menu. Use the file
browser to find a labeler data file produced by the
`make_labeler_data_file` program. If you already have a set of labels
for this file you can open it for editing by using the "open label
file** option in the file menu. Otherwise, a default label file will be
produced. You can use the save menu items to save your label file as
you go.

*NOTE*: Saving will overwite the file that exists. Make sure you are
saving to the right file before clicking the save buttons.

The upper area contains playback tools. The top two widgets control
playback speed. With higher numbers being faster. The next two widgets
control the current frame being displayed.

The player buttons below will change the playback mode. The rewind
button will play backwards at the specified playback speed. The fast
forward button will do the same, but forwards. The step back and step
forward buttons will move one frame backwards or forwards
respectively. The pause button will pause playback at the current
frame.

Below the player, is a tab for each type of event
labeling. Instantaneous events show the label editing for the current
frame according to the player. Duration events have a list on the left
hand side. You can add and delete new duration events using the
buttons above the list. Click on an item in the list to edit it or
select it for deletion. A screenshot is shown below.

![Duration Event
Editing](/screenshots/duration_event_editing.png?raw=true "Duration
Event Editing")

## Labeler Data File

Produced by the `make_labeler_data_file` utility (see above).

The file is a binary format. All numeric values are written in
BigEndian order. Each file starts with the following header:

``` text
1: String - File type ("SSL_LABELER_DATA")
2: uint32 - Log file format version
```

Then the actual frame messages follow. Each message starts with the
message length, and then a serialized set of bytes for a Protobuf
`log_labeler_data::LabelerFrameGroup` message. See the
`log_labeler_data.proto` file for the exact message definition.

``` text
1: uint32 - Message length
2: bytes - Binary log_labeler_data::LabelerFrameGroup protobuf message
```

This file is designed to be easily seekable to specific messages. As
such, the final bytes in the file contain metadata about where each
message in the file starts. The last 4 bytes are the metadata message
size. Then the proceeding byte string corresponds to a binary protobuf
`log_labeler_data::LabelerMetadata` message.

``` text
1: bytes - Binary log_labeler_data::LabelerMetadata protobuf message
2: uint32 - size of preceding metadata bytes
```

## Label File

This is the file produced by the `label_data` gui and scored by the
`score_label_file` utility. It is just a binary protobuf
`log_labels::Label` message. It contains a list of all event labels
specified in the technical challenge rules.

*Note*: that the duration event labels are sorted by start_frame. If
you do not sort your output by start time you will receive an
inaccurate score from the scoring program.
