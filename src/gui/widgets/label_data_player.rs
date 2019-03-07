use crate::labeler::player::Player;
use crate::labeler::reader::LabelerDataReader;
use crossbeam::channel::{bounded, unbounded, TryRecvError};
use imgui::*;
use std::io::{Read, Seek};
use std::{cmp, thread, time};

const DEFAULT_FRAME_DURATION: time::Duration = time::Duration::from_millis(16);

// icon font codes
const FA_REWIND: &str = "\u{f04a}";
const FA_STEP_BACK: &str = "\u{f048}";
const FA_PAUSE: &str = "\u{f04c}";
const FA_STEP_FORWARD: &str = "\u{f051}";
const FA_FAST_FORWARD: &str = "\u{f04e}";

#[derive(Debug, Clone, Copy)]
pub enum PlayState {
    Paused,
    Forward,
    Backward,
}

enum PlayerThreadCommand {
    StopThread,
    GetState,
    ChangePlaybackSpeed(f32),
    ChangePlayState(PlayState),
    ChangeFrame(usize),
}

#[derive(Debug, Clone)]
struct PlayerThreadState {
    play_state: PlayState,
    playback_speed: f32,
    curr_frame: usize,
}

impl Default for PlayerThreadState {
    fn default() -> Self {
        Self {
            play_state: PlayState::Paused,
            playback_speed: 1.0,
            curr_frame: 0,
        }
    }
}

pub struct LabelDataPlayer {
    _player_thread: thread::JoinHandle<()>,
    command_channel: crossbeam::channel::Sender<PlayerThreadCommand>,
    state_channel: crossbeam::channel::Receiver<PlayerThreadState>,

    latest_thread_state: PlayerThreadState,
    num_frames: usize,
}

impl LabelDataPlayer {
    pub fn new<T: Read + Seek + Send + 'static>(reader: T) -> LabelDataPlayer {
        let reader = LabelerDataReader::new(reader).unwrap();
        let player = Player::new(reader);

        let (command_sender, command_receiver) = unbounded();
        let (state_sender, state_receiver) = bounded(1);

        let num_frames = player.len();

        let player_thread = PlayerThread::start(command_receiver, state_sender, player);

        LabelDataPlayer {
            _player_thread: player_thread,
            command_channel: command_sender,
            state_channel: state_receiver,
            latest_thread_state: Default::default(),
            num_frames,
        }
    }

    pub fn len(&self) -> usize {
        self.num_frames
    }

    pub fn curr_frame(&self) -> usize {
        self.latest_thread_state.curr_frame
    }

    pub fn build<'ui>(&mut self, ui: &Ui<'ui>) {
        ui.text("Playback Speed");
        let playback_speed_input_changed = ui
            .input_float(
                im_str!("##Playback Speed Input"),
                &mut self.latest_thread_state.playback_speed,
            )
            .enter_returns_true(true)
            .build();
        let playback_speed_slider_changed = ui
            .slider_float(
                im_str!("##Playback Speed Slider"),
                &mut self.latest_thread_state.playback_speed,
                0.0,
                10.0,
            )
            .build();
        if playback_speed_input_changed || playback_speed_slider_changed {
            self.command_channel
                .send(PlayerThreadCommand::ChangePlaybackSpeed(
                    self.latest_thread_state.playback_speed,
                ))
                .unwrap();
        }

        let mut curr_frame = self.latest_thread_state.curr_frame as i32;
        ui.text("Current Frame");
        let frame_input_changed = ui
            .input_int(im_str!("##Current Frame Input"), &mut curr_frame)
            .enter_returns_true(true)
            .build();
        let frame_slider_changed = ui
            .slider_int(
                im_str!("##Current Frame Slider"),
                &mut curr_frame,
                0,
                self.num_frames as i32,
            )
            .build();
        if frame_input_changed || frame_slider_changed {
            self.latest_thread_state.curr_frame = curr_frame as usize;
            self.command_channel
                .send(PlayerThreadCommand::ChangeFrame(
                    self.latest_thread_state.curr_frame,
                ))
                .unwrap();
        }

        let x_pos = 0.0;
        let button_text_size = ui.calc_text_size(im_str!("{}", FA_REWIND), true, 100.0);
        let button_x_padding = 2.0 * ui.imgui().style().item_spacing.x;

        if ui.button(im_str!("{}", FA_REWIND), (0.0, 0.0)) {
            self.command_channel
                .send(PlayerThreadCommand::ChangePlayState(PlayState::Backward))
                .unwrap();
        }

        let x_pos = x_pos + button_text_size.x + 2.0 * button_x_padding;
        ui.same_line(x_pos);
        if ui.button(im_str!("{}", FA_STEP_BACK), (0.0, 0.0)) {
            self.command_channel
                .send(PlayerThreadCommand::ChangePlayState(PlayState::Paused))
                .unwrap();
            if self.latest_thread_state.curr_frame > 0 {
                self.latest_thread_state.curr_frame = self.latest_thread_state.curr_frame - 1;
            }
            self.command_channel
                .send(PlayerThreadCommand::ChangeFrame(
                    self.latest_thread_state.curr_frame,
                ))
                .unwrap();
        }

        let x_pos = x_pos + button_text_size.x + 2.0 * button_x_padding;
        ui.same_line(x_pos);
        if ui.button(im_str!("{}", FA_PAUSE), (0.0, 0.0)) {
            self.command_channel
                .send(PlayerThreadCommand::ChangePlayState(PlayState::Paused))
                .unwrap();
        }

        let x_pos = x_pos + button_text_size.x + 2.0 * button_x_padding;
        ui.same_line(x_pos);
        if ui.button(im_str!("{}", FA_STEP_FORWARD), (0.0, 0.0)) {
            self.command_channel
                .send(PlayerThreadCommand::ChangePlayState(PlayState::Paused))
                .unwrap();
            self.latest_thread_state.curr_frame =
                cmp::min(self.latest_thread_state.curr_frame + 1, self.num_frames);
            self.command_channel
                .send(PlayerThreadCommand::ChangeFrame(
                    self.latest_thread_state.curr_frame,
                ))
                .unwrap();
        }

        let x_pos = x_pos + button_text_size.x + 2.0 * button_x_padding;
        ui.same_line(x_pos);
        if ui.button(im_str!("{}", FA_FAST_FORWARD), (0.0, 0.0)) {
            self.command_channel
                .send(PlayerThreadCommand::ChangePlayState(PlayState::Forward))
                .unwrap();
        }

        // sync state with thread
        self.command_channel
            .send(PlayerThreadCommand::GetState)
            .unwrap();
        self.latest_thread_state = self.state_channel.recv().unwrap();
    }
}

impl Drop for LabelDataPlayer {
    fn drop(&mut self) {
        self.command_channel
            .send(PlayerThreadCommand::StopThread)
            .unwrap();
    }
}

struct PlayerThread<T: Read + Seek + Send + 'static> {
    command_channel: crossbeam::channel::Receiver<PlayerThreadCommand>,
    state_channel: crossbeam::channel::Sender<PlayerThreadState>,
    player: Player<T>,
    play_state: PlayState,
    curr_frame: usize,
    playback_speed: f32,
    last_frame_time: Option<time::Instant>,
    frame_duration: time::Duration,
}

impl<T: Read + Seek + Send + 'static> PlayerThread<T> {
    pub fn start(
        command_channel: crossbeam::channel::Receiver<PlayerThreadCommand>,
        state_channel: crossbeam::channel::Sender<PlayerThreadState>,
        player: Player<T>,
    ) -> thread::JoinHandle<()> {
        let player_thread = PlayerThread {
            command_channel,
            state_channel,
            player,
            play_state: PlayState::Paused,
            curr_frame: 0,
            playback_speed: 1.0,
            last_frame_time: None,
            frame_duration: DEFAULT_FRAME_DURATION,
        };

        thread::spawn(|| player_thread.player_thread_func())
    }

    fn player_thread_func(mut self) {
        loop {
            // check for new command
            match self.command_channel.try_recv() {
                Ok(command) => match command {
                    PlayerThreadCommand::StopThread => {
                        break;
                    }
                    PlayerThreadCommand::GetState => {
                        let thread_state = PlayerThreadState {
                            play_state: self.play_state,
                            playback_speed: self.playback_speed,
                            curr_frame: self.curr_frame,
                        };
                        self.state_channel.send(thread_state).unwrap();
                    }
                    PlayerThreadCommand::ChangePlayState(play_state) => {
                        self.play_state = play_state;
                    }
                    PlayerThreadCommand::ChangePlaybackSpeed(playback_speed) => {
                        self.playback_speed = playback_speed;

                        let frame_duration_micros = ((DEFAULT_FRAME_DURATION.as_micros() as f64)
                            / (self.playback_speed as f64))
                            as u64;

                        self.frame_duration = time::Duration::from_micros(frame_duration_micros);
                    }
                    PlayerThreadCommand::ChangeFrame(frame) => {
                        self.curr_frame = frame;
                        self.player.play_frame(frame);
                    }
                },
                Err(error) => match error {
                    TryRecvError::Empty => {
                        // do nothing
                    }
                    TryRecvError::Disconnected => {
                        // this should never happen, but it effectively
                        // means the main player object is gone and this
                        // thread should have been signaled to stop.
                        break;
                    }
                },
            };

            match self.play_state {
                PlayState::Forward => match self.last_frame_time {
                    Some(ref last_frame_time) => {
                        let elapsed_time = time::Instant::now() - *last_frame_time;
                        if elapsed_time > self.frame_duration {
                            self.play_next_frame();
                        }
                    }
                    None => self.play_next_frame(),
                },
                PlayState::Backward => match self.last_frame_time {
                    Some(ref last_frame_time) => {
                        let elapsed_time = time::Instant::now() - *last_frame_time;
                        if elapsed_time > self.frame_duration {
                            self.play_prev_frame();
                        }
                    }
                    None => self.play_prev_frame(),
                },
                PlayState::Paused => self.last_frame_time = None,
            };

            // set this to playback speed time? Might be an issue with
            // widget state being out of sync
            thread::sleep(time::Duration::from_millis(1));
        }
    }

    fn play_next_frame(&mut self) {
        self.last_frame_time = Some(time::Instant::now());
        self.curr_frame = cmp::min(self.curr_frame + 1, self.player.len());
        self.player.play_frame(self.curr_frame);
    }

    fn play_prev_frame(&mut self) {
        self.last_frame_time = Some(time::Instant::now());
        if self.curr_frame > 0 {
            self.curr_frame = self.curr_frame - 1;
        }
        self.player.play_frame(self.curr_frame);
    }
}
