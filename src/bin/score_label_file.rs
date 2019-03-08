use clap::{App, Arg};
use protobuf;
use ssl_log_tools::protos::log_labels;
use std::cmp;
use std::fs;

fn main() {
    let matches = App::new("Score a label file.")
        .version("1.0")
        .author("Devin Schwab <dschwab@andrew.cmu.edu>")
        .about("Score a log file for the SSL 2019 Technical Challenge")
        .arg(
            Arg::with_name("GROUND_TRUTH")
                .help("Path to ground truth label file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("PREDICTED")
                .help("Path to predicted label file")
                .required(true)
                .index(2),
        )
        .get_matches();

    let mut ground_truth = fs::File::open(matches.value_of("GROUND_TRUTH").unwrap())
        .expect("Failed to open ground truth label file");
    let mut predicted = fs::File::open(matches.value_of("PREDICTED").unwrap())
        .expect("Failed to open predicted label file");

    let ground_truth_labels: log_labels::Labels = protobuf::parse_from_reader(&mut ground_truth)
        .expect("Failed to parse ground truth label file");
    let predicted_labels: log_labels::Labels =
        protobuf::parse_from_reader(&mut predicted).expect("Failed to parse predicted label file");

    let dribbling_score = score_dribbling(
        ground_truth_labels.get_dribbling_labels(),
        predicted_labels.get_dribbling_labels(),
    );
    let ball_possession_score = score_ball_possession(
        ground_truth_labels.get_ball_possession_labels(),
        predicted_labels.get_ball_possession_labels(),
    );
    let passing_score = score_passing(
        ground_truth_labels.get_passing_labels(),
        predicted_labels.get_passing_labels(),
    );
    let goal_shot_score = score_goal_shot(
        ground_truth_labels.get_goal_shot_labels(),
        predicted_labels.get_goal_shot_labels(),
    );

    println!("Dribbling Score: {}", dribbling_score);
    println!("Ball Possession Score: {}", ball_possession_score);
    println!("Passing Score: {}", passing_score);
    println!("Goal Shot Score: {}", goal_shot_score);
}

fn score_dribbling(
    ground_truth_labels: &[log_labels::DribblingLabel],
    predicted_labels: &[log_labels::DribblingLabel],
) -> f64 {
    let mut score: f64 = 0.0;
    for (ground_truth_label, predicted_label) in
        ground_truth_labels.iter().zip(predicted_labels.iter())
    {
        if ground_truth_label.get_is_dribbling() == predicted_label.get_is_dribbling() {
            score += 1.0;

            if ground_truth_label.get_robot_id() == predicted_label.get_robot_id() {
                score += 0.5;
            }

            if ground_truth_label.get_team() == predicted_label.get_team() {
                score += 0.5;
            }
        }
    }

    score
}

fn score_ball_possession(
    ground_truth_labels: &[log_labels::BallPossessionLabel],
    predicted_labels: &[log_labels::BallPossessionLabel],
) -> f64 {
    let mut score: f64 = 0.0;
    for (ground_truth_label, predicted_label) in
        ground_truth_labels.iter().zip(predicted_labels.iter())
    {
        if ground_truth_label.get_state() == predicted_label.get_state() {
            score += 1.0;

            if ground_truth_label.get_robot_id() == predicted_label.get_robot_id() {
                score += 0.5;
            }
        }
    }
    score
}

fn calc_iou(span_a: (u64, u64), span_b: (u64, u64)) -> f64 {
    // get the union
    let union_start = cmp::min(span_a.0, span_b.0);
    let union_end = cmp::max(span_a.1, span_b.1);

    let union = union_end - union_start;
    // this shouldn't really happen, but if it does happen in both the
    // ground truth and predicted, count it as a perfect match
    if union == 0 {
        return 1.0;
    }
    let union = union as f64;

    // get the overlap
    let intersect: f64 = if span_a.0 > span_b.1 || span_b.0 > span_a.1 {
        0.0
    } else {
        let intersect_start = cmp::max(span_a.0, span_b.0);
        let intersect_end = cmp::min(span_a.1, span_b.1);

        (intersect_end - intersect_start) as f64
    };

    intersect / union
}

fn score_passing(
    ground_truth_labels: &[log_labels::PassingLabel],
    predicted_labels: &[log_labels::PassingLabel],
) -> f64 {
    let mut score: f64 = 0.0;
    for (ground_truth_label, predicted_label) in
        ground_truth_labels.iter().zip(predicted_labels.iter())
    {
        let ground_truth_span = (
            ground_truth_label.get_start_frame(),
            ground_truth_label.get_end_frame(),
        );
        let predicted_span = (
            predicted_label.get_start_frame(),
            predicted_label.get_end_frame(),
        );
        score += calc_iou(ground_truth_span, predicted_span);

        if ground_truth_label.get_successful() == predicted_label.get_successful() {
            score += 0.5;
        }

        if ground_truth_label.get_passer_id() == predicted_label.get_passer_id() {
            score += 0.5;
        }

        if ground_truth_label.get_passer_team() == predicted_label.get_passer_team() {
            score += 0.5;
        }

        if ground_truth_label.get_receiver_id() == predicted_label.get_receiver_id() {
            score += 0.5;
        }
    }
    score
}

fn score_goal_shot(
    ground_truth_labels: &[log_labels::GoalShotLabel],
    predicted_labels: &[log_labels::GoalShotLabel],
) -> f64 {
    let mut score: f64 = 0.0;
    for (ground_truth_label, predicted_label) in
        ground_truth_labels.iter().zip(predicted_labels.iter())
    {
        let ground_truth_span = (
            ground_truth_label.get_start_frame(),
            ground_truth_label.get_end_frame(),
        );
        let predicted_span = (
            predicted_label.get_start_frame(),
            predicted_label.get_end_frame(),
        );
        score += calc_iou(ground_truth_span, predicted_span);

        if ground_truth_label.get_successful() == predicted_label.get_successful() {
            score += 0.5;
        }

        if ground_truth_label.get_shooter_id() == predicted_label.get_shooter_id() {
            score += 0.5;
        }

        if ground_truth_label.get_shooter_team() == predicted_label.get_shooter_team() {
            score += 0.5;
        }
    }
    score
}
