use clap::{App, Arg};
use ndarray::prelude::*;
use petgraph::algo;
use petgraph::prelude::*;
use protobuf;
use ssl_rust_tools::protos::log_labels;
use std::cmp;
use std::collections::HashMap;
use std::fs;

const UP: u8 = 0b1;
const DIAG: u8 = 0b10;
const LEFT: u8 = 0b100;

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

            // only score sub-fields if labeled as dribbling in ground truth
            if ground_truth_label.get_is_dribbling() {
                if ground_truth_label.get_robot_id() == predicted_label.get_robot_id() {
                    score += 0.5;
                }

                if ground_truth_label.get_team() == predicted_label.get_team() {
                    score += 0.5;
                }
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

            // only score if yellow/blue in possession
            if ground_truth_label.get_state() != log_labels::BallPossessionLabel_State::NONE {
                if ground_truth_label.get_robot_id() == predicted_label.get_robot_id() {
                    score += 0.5;
                }
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

/// Uses Smith–Waterman alignment algorithm to match duration event labels
///
/// See https://en.wikipedia.org/wiki/Smith%E2%80%93Waterman_algorithm#Algorithm
///     https://www.ks.uiuc.edu/Training/SumSchool/materials/sources/tutorials/07-bioinformatics/seqlab-html/node6.html#table:dynMat2
///
/// Currently the penalty for a misalignment is 0. So this should find
/// the best matching assignment regardless of necessary shift
/// distance.
///
/// Once matches are found IOU and label matching proceeds as normal
/// between the matched labels.
fn match_labels(gt_spans: &[(u64, u64)], pred_spans: &[(u64, u64)]) -> Vec<(usize, usize)> {
    let mut h_matrix: Array2<f64> = Array::zeros((pred_spans.len() + 1, gt_spans.len() + 1));
    let mut dir_graph = Graph::<(usize, usize), u8>::new();
    let mut dir_nodes: HashMap<(usize, usize), NodeIndex> = HashMap::new();
    for i in 0..h_matrix.shape()[0] {
        let node = dir_graph.add_node((i, 0));
        dir_nodes.insert((i, 0), node);
    }
    for j in 0..h_matrix.shape()[1] {
        let node = dir_graph.add_node((0, j));
        dir_nodes.insert((0, j), node);
    }

    for i in 1..h_matrix.shape()[0] {
        for j in 1..h_matrix.shape()[1] {
            let gt_span = (gt_spans[j - 1].0 as u64, gt_spans[j - 1].1 as u64);
            let pred_span = (pred_spans[i - 1].0 as u64, pred_spans[i - 1].1 as u64);
            let iou = calc_iou(gt_span, pred_span);

            let diag_score = h_matrix[[i - 1, j - 1]] + iou;
            let up_score = h_matrix[[i - 1, j]];
            let left_score = h_matrix[[i, j - 1]];

            let node = dir_graph.add_node((i, j));
            dir_nodes.insert((i, j), node);

            let best_score = diag_score.max(up_score).max(left_score);
            if (best_score - up_score).abs() < 1e-6 {
                let parent_node = dir_nodes.get(&(i - 1, j)).unwrap();
                dir_graph.add_edge(node, *parent_node, UP);
            }
            if (best_score - diag_score).abs() < 1e-6 {
                let parent_node = dir_nodes.get(&(i - 1, j - 1)).unwrap();
                dir_graph.add_edge(node, *parent_node, DIAG);
            }
            if (best_score - left_score).abs() < 1e-6 {
                let parent_node = dir_nodes.get(&(i, j - 1)).unwrap();
                dir_graph.add_edge(node, *parent_node, LEFT);
            }
            h_matrix[[i, j]] = best_score;
        }
    }

    // find max element for traceback start position
    let mut highest_value: f64 = 0.0;
    let mut highest_pos = (0, 0);
    for i in 0..h_matrix.shape()[0] {
        for j in 0..h_matrix.shape()[1] {
            if h_matrix[[i, j]] >= highest_value {
                highest_value = h_matrix[[i, j]];
                highest_pos = (i, j);
            }
        }
    }

    // println!("h_matrix: {:#?}", h_matrix);
    // // println!("dir_matrix: {:#?}", dir_matrix);
    // println!("Highest value: {}", highest_value);
    // println!("Highest pos: {:#?}", highest_pos);

    let start_node = dir_nodes.get(&highest_pos).unwrap();
    let path = algo::astar(
        &dir_graph,
        *start_node,
        |finish| {
            let index = dir_graph.node_weight(finish).unwrap();
            let score = h_matrix[*index];

            score.abs() < 1e-6
        },
        |edge| match *edge.weight() {
            DIAG => 0,
            UP => 1,
            LEFT => 1,
            _ => 2,
        },
        |_| 0,
    )
    .unwrap();

    let mut matches = Vec::<(usize, usize)>::new();
    let mut prev_node = path.1[0];
    for node in path.1.iter().skip(1) {
        let prev_index = dir_graph.node_weight(prev_node).unwrap();
        let curr_index = dir_graph.node_weight(*node).unwrap();
        if prev_index.0 - curr_index.0 == 1 && prev_index.1 - curr_index.1 == 1 {
            matches.push((prev_index.1 - 1, prev_index.0 - 1));
        }
        prev_node = *node;
    }

    matches
}

fn score_passing(
    ground_truth_labels: &[log_labels::PassingLabel],
    predicted_labels: &[log_labels::PassingLabel],
) -> f64 {
    let mut gt_spans = Vec::<(u64, u64)>::new();
    for gt_label in ground_truth_labels.iter() {
        gt_spans.push((gt_label.get_start_frame(), gt_label.get_end_frame()));
    }
    let mut pred_spans = Vec::<(u64, u64)>::new();
    for pred_label in predicted_labels.iter() {
        pred_spans.push((pred_label.get_start_frame(), pred_label.get_end_frame()));
    }
    let label_matches = match_labels(&pred_spans, &gt_spans);

    let mut score: f64 = 0.0;
    for (pred_index, gt_index) in label_matches {
        let pred_label = &predicted_labels[pred_index];
        let gt_label = &ground_truth_labels[gt_index];

        let gt_span = gt_spans[gt_index];
        let pred_span = pred_spans[pred_index];
        score += calc_iou(gt_span, pred_span);

        if gt_label.get_successful() == pred_label.get_successful() {
            score += 0.5;
        }

        if gt_label.get_passer_id() == pred_label.get_passer_id() {
            score += 0.5;
        }

        if gt_label.get_passer_team() == pred_label.get_passer_team() {
            score += 0.5;
        }

        if gt_label.get_receiver_id() == pred_label.get_receiver_id() {
            score += 0.5;
        }
    }
    score
}

fn score_goal_shot(
    ground_truth_labels: &[log_labels::GoalShotLabel],
    predicted_labels: &[log_labels::GoalShotLabel],
) -> f64 {
    let mut gt_spans = Vec::<(u64, u64)>::new();
    for gt_label in ground_truth_labels.iter() {
        gt_spans.push((gt_label.get_start_frame(), gt_label.get_end_frame()));
    }
    let mut pred_spans = Vec::<(u64, u64)>::new();
    for pred_label in predicted_labels.iter() {
        pred_spans.push((pred_label.get_start_frame(), pred_label.get_end_frame()));
    }
    let label_matches = match_labels(&pred_spans, &gt_spans);

    let mut score: f64 = 0.0;
    for (pred_index, gt_index) in label_matches {
        let pred_label = &predicted_labels[pred_index];
        let gt_label = &ground_truth_labels[gt_index];

        let gt_span = gt_spans[gt_index];
        let pred_span = pred_spans[pred_index];
        score += calc_iou(gt_span, pred_span);

        if gt_label.get_successful() == pred_label.get_successful() {
            score += 0.5;
        }

        if gt_label.get_shooter_id() == pred_label.get_shooter_id() {
            score += 0.5;
        }

        if gt_label.get_shooter_team() == pred_label.get_shooter_team() {
            score += 0.5;
        }
    }
    score
}
