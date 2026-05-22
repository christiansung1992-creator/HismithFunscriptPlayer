// src/buttplug/funscript_utils.rs

//! Funscript processing module
//!
//! This module handles processing of funscript files which contain synchronized motion
//! data for videos. It provides utilities to:
//! - Parse funscript data structures
//! - Calculate motion intensities from discrete action sets
//! - Interpolate between motion points
//! - Optimize motion data for real-time playback
//!
//! Conventions and units:
//! - Time is expressed in milliseconds (u64).
//! - Position values are floating point in the range 0.0 .. 100.0.
//! - Many helpers assume the input funscript uses "binary" extremes (0 or 100) when
//!   deriving speed-based intensity. Functions validate and document when this is required.
//! - Intensity values returned by processing functions are in the same 0.0 .. 100.0 range.

use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

/// Represents a single motion action at a specific timestamp
///
/// Actions contain a timestamp (`at`) in milliseconds and a position (`pos`)
/// value between 0.0 and 100.0 representing the motion position. The struct is
/// serializable to/from standard funscript JSON representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    /// Timestamp in milliseconds when this action occurs
    #[serde(rename = "at")]
    pub at: u64,
    /// Position value between 0.0 (min) and 100.0 (max)
    #[serde(rename = "pos")]
    pub pos: f64,
}

/// Collection of motion actions forming a complete funscript
///
/// Contains an ordered sequence of actions that define the motion pattern over time.
/// Fields map directly to the relevant funscript metadata fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunscriptData {
    /// Vector of actions in chronological order
    pub actions: Vec<Action>,

    /// Funscript version string (defaults to "1.0")
    #[serde(default = "default_version")]
    pub version: String,

    /// Whether positions are inverted
    #[serde(default)]
    pub inverted: bool,

    /// Maximum range value used by the script (default 100)
    #[serde(default = "default_range")]
    pub range: u32,

    /// Optional arbitrary metadata from the original file
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

fn default_version() -> String {
    "1.0".to_string()
}

fn default_range() -> u32 {
    100
}

impl Default for FunscriptData {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            version: default_version(),
            inverted: false,
            range: default_range(),
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
/// Simple serializable point used by the calibration API.
/// Maps a BPM value to a target intensity (0..100).
pub struct BpmIntensityPoint {
    pub bpm: f64,
    pub intensity: f64,
}

/// BPM -> intensity calibration table
///
/// This table defines a piecewise-linear mapping used by the intensity
/// generation pipeline. BPM values outside the table range are clamped
/// to the nearest endpoints.
pub const BPM_INTENSITY_MAP: [(f64, f64); 11] = [
    (0.0, 0.0),
    (42.0, 10.0),
    (66.0, 20.0),
    (90.0, 30.0),
    (116.0, 40.0),
    (140.0, 50.0),
    (160.0, 60.0),
    (182.0, 70.0),
    (218.0, 80.0),
    (245.0, 90.0),
    (270.0, 100.0),
];

/// Returns the calibration mapping as a JSON-serializable Vec
///
/// This is used by the HTTP calibration endpoint to return the active
/// BPM -> intensity mapping to clients.
pub fn get_bpm_intensity_mapping() -> Vec<BpmIntensityPoint> {
    BPM_INTENSITY_MAP
        .iter()
        .map(|(b, i)| BpmIntensityPoint {
            bpm: *b,
            intensity: *i,
        })
        .collect()
}

/// Calculates the interpolated position at a given time between two actions
///
/// Uses linear interpolation to determine the position at any timestamp
/// between two known action points. If one side is None, the function will
/// behave as a step to the available endpoint. This helper expects `at`
/// timestamps to be expressed in milliseconds.
///
/// # Arguments
/// * `a0` - The previous action (or None if before first action)
/// * `a1` - The next action (or None if after last action)
/// * `time` - The timestamp to interpolate at
///
/// # Returns
/// * `f64` - The interpolated position value (0.0 .. 100.0)
fn interpolate_position(a0: Option<&Action>, a1: Option<&Action>, time: u64) -> f64 {
    match (a0, a1) {
        (None, None) => 0.0,
        (None, Some(act1)) => act1.pos,
        (Some(act0), None) => act0.pos,
        (Some(act0), Some(act1)) => {
            if time <= act0.at {
                return act0.pos;
            }
            if time >= act1.at {
                return act1.pos;
            }
            if act0.at == act1.at {
                return act0.pos;
            }

            let time_fraction = (time - act0.at) as f64 / (act1.at - act0.at) as f64;
            act0.pos + (act1.pos - act0.pos) * time_fraction
        }
    }
}

/// Optimizes action data by combining consecutive identical positions
///
/// Reduces the number of actions by averaging timestamps of consecutive
/// actions with the same position value within a specified time window.
///
/// # Arguments
/// * `actions` - Vector of actions to optimize (will be replaced with condensed set)
/// * `max_gap_ms` - Maximum time gap in milliseconds to consider positions identical
fn condense_identical_positions(actions: &mut Vec<Action>, max_gap_ms: u64) {
    if actions.is_empty() {
        return;
    }

    let mut condensed = Vec::new();
    let mut group = vec![actions[0].clone()];

    for a in actions.iter().skip(1) {
        if (a.pos == group.last().unwrap().pos) && (a.at - group.last().unwrap().at <= max_gap_ms) {
            group.push(a.clone());
        } else {
            if group.len() > 1 {
                // Average timestamps for grouped actions
                let avg_at = group.iter().map(|x| x.at as u128).sum::<u128>() / group.len() as u128;
                condensed.push(Action {
                    at: avg_at as u64,
                    pos: group[0].pos,
                });
            } else {
                condensed.push(group[0].clone());
            }
            group = vec![a.clone()];
        }
    }

    // Handle the last group
    if group.len() > 1 {
        let avg_at = group.iter().map(|x| x.at as u128).sum::<u128>() / group.len() as u128;
        condensed.push(Action {
            at: avg_at as u64,
            pos: group[0].pos,
        });
    } else {
        condensed.push(group[0].clone());
    }

    *actions = condensed;
}

/// Calculates continuous intensity values from discrete motion actions
///
/// Processes raw motion data to generate a continuous intensity curve that
/// represents the speed and amplitude of movements. The function computes
/// summed absolute percent changes inside a sliding window and converts that
/// rate into an effective BPM which is then mapped to an intensity value
/// using the BPM_INTENSITY_MAP. The conversion derivation:
///   - A full thrust is treated as 200% position change (0 -> 100 -> 0)
///   - percent_per_ms -> BPM uses factor 300.0 (see calculate_window_intensity)
///
/// Important notes:
/// - The input actions are expected to be mostly binary (positions 0.0 or 100.0).
///   The function will early-return if any action has an intermediate value.
/// - `sample_rate_ms` determines the spacing of output samples. Output timestamps
///   are rounded to multiples of this value.
/// - Returned actions contain intensity values in the range 0.0 .. 100.0
///
/// # Arguments
/// * `actions` - Mutable slice of motion actions to process (will be cloned and sorted)
/// * `sample_rate_ms` - How often to sample the intensity (milliseconds)
/// * `window_radius_ms` - Size of the moving analysis window (milliseconds)
///
/// # Returns
/// * `Vec<Action>` - Vector of actions containing calculated intensities with timestamps
pub fn calculate_thrust_intensity_by_scaled_speed(
    actions: &mut [Action],
    sample_rate_ms: u64,
    window_radius_ms: u64,
) -> Vec<Action> {
    if actions.len() < 2 {
        return Vec::new();
    }

    // Validate input positions
    if let Some(invalid_action) = actions.iter().find(|a| a.pos != 0.0 && a.pos != 100.0) {
        eprintln!(
            "Error: Invalid position value {} at time {}ms. Valid values are 0 or 100.",
            invalid_action.pos, invalid_action.at
        );
        return Vec::new();
    }

    // Initialize processing
    actions.sort_by_key(|a| a.at);
    let mut actions_vec = actions.to_vec();
    condense_identical_positions(&mut actions_vec, 200);

    let mut output_actions = Vec::new();
    let min_time = actions_vec.first().unwrap().at;
    let max_time = actions_vec.last().unwrap().at;

    // Configuration constants
    const MAX_INCREASE_PER_SEC: f64 = 40.0; // Maximum intensity increase per second
    const SLOW_ALPHA: f64 = 0.6; // Smoothing factor
    let max_increase_per_ms = MAX_INCREASE_PER_SEC / 1000.0;

    // Add initial zero point if needed
    if min_time > 0 {
        output_actions.push(Action { at: 0, pos: 0.0 });
    }

    // Processing state
    let mut t = 0;
    let mut previous_intensity = 0.0;
    let mut previous_smooth = 0.0;

    // Main processing loop
    while t <= max_time {
        let window_start = max(0, t.saturating_sub(window_radius_ms));
        let window_end = min(max_time, t + window_radius_ms);
        let window_duration_ms = window_end.saturating_sub(window_start);

        // Calculate raw intensity within window
        let mut raw_intensity = if window_duration_ms > 0 {
            calculate_window_intensity(&actions_vec, window_start, window_end, window_duration_ms)
        } else {
            0.0
        };

        // Apply rate limiting
        if sample_rate_ms > 0 {
            let max_inc = max_increase_per_ms * sample_rate_ms as f64;
            if raw_intensity > previous_intensity + max_inc {
                raw_intensity = previous_intensity + max_inc;
            }
        }

        // Apply smoothing and create output action
        let rounded_time = ((t as f64 / sample_rate_ms as f64).round() as u64) * sample_rate_ms;
        let smooth_intensity = previous_smooth + SLOW_ALPHA * (raw_intensity - previous_smooth);
        let final_intensity = raw_intensity.max(smooth_intensity);

        output_actions.push(Action {
            at: rounded_time,
            pos: final_intensity,
        });

        // Update state
        previous_smooth = smooth_intensity;
        previous_intensity = final_intensity;
        t += sample_rate_ms;
        if sample_rate_ms == 0 {
            break;
        }
    }

    output_actions
}

/// Maps measured BPM to calibrated intensity (0..100) using piecewise linear interpolation.
///
/// The function performs a simple piecewise-linear interpolation across the
/// BPM_INTENSITY_MAP calibration table. Non-finite inputs are treated as 0,
/// and values outside the table bounds are clamped to the nearest endpoint.
///
/// # Arguments
/// * `bpm` - Beats-per-minute equivalent derived from motion velocity
///
/// # Returns
/// * `f64` - Intensity in the range 0.0 .. 100.0
fn map_bpm_to_intensity(bpm: f64) -> f64 {
    if !bpm.is_finite() {
        return 0.0;
    }
    if bpm <= BPM_INTENSITY_MAP[0].0 {
        return BPM_INTENSITY_MAP[0].1;
    }
    if bpm >= BPM_INTENSITY_MAP[BPM_INTENSITY_MAP.len() - 1].0 {
        return BPM_INTENSITY_MAP[BPM_INTENSITY_MAP.len() - 1].1;
    }
    for i in 0..(BPM_INTENSITY_MAP.len() - 1) {
        let (b0, i0) = BPM_INTENSITY_MAP[i];
        let (b1, i1) = BPM_INTENSITY_MAP[i + 1];
        if bpm >= b0 && bpm <= b1 {
            let t = (bpm - b0) / (b1 - b0);
            return i0 + (i1 - i0) * t;
        }
    }
    0.0
}

/// Helper function to calculate intensity within a time window
///
/// This routine:
/// 1. Constructs a list of position samples across the specified window,
///    inserting interpolated boundary samples at window_start and window_end.
/// 2. Sums absolute percent position changes between successive samples.
/// 3. Converts the sum (percent per ms) to an equivalent BPM using a constant
///    scaling factor and maps that BPM to an intensity value via the calibration table.
///
/// # Arguments
/// * `actions` - Source actions (assumed sorted by `at`)
/// * `window_start` - Window start time in ms (inclusive)
/// * `window_end` - Window end time in ms (inclusive)
/// * `window_duration_ms` - window_end - window_start (precomputed)
///
/// # Returns
/// * `f64` - Intensity value (0.0 .. 100.0)
fn calculate_window_intensity(
    actions: &[Action],
    window_start: u64,
    window_end: u64,
    window_duration_ms: u64,
) -> f64 {
    // Find boundary actions
    let start_idx = actions
        .iter()
        .rposition(|a| a.at <= window_start)
        .unwrap_or(0);
    let start_action = &actions[start_idx];
    let end_idx = actions
        .iter()
        .position(|a| a.at >= window_end)
        .unwrap_or(actions.len() - 1);
    let end_action = &actions[end_idx];

    // Build points list with interpolated boundaries
    let mut pts = Vec::new();
    pts.push(Action {
        at: window_start,
        pos: interpolate_position(Some(start_action), actions.get(start_idx + 1), window_start),
    });

    // Add intermediate points
    pts.extend(
        actions
            .iter()
            .filter(|a| a.at > window_start && a.at < window_end)
            .cloned(),
    );

    // Add end point
    let prev_for_end = actions[..end_idx]
        .iter()
        .rev()
        .find(|a| a.at < window_end)
        .or(Some(start_action));
    pts.push(Action {
        at: window_end,
        pos: interpolate_position(prev_for_end, Some(end_action), window_end),
    });

    // Calculate summed absolute percent change within the window
    let raw_sum_percent = pts
        .windows(2)
        .filter(|w| w[1].at > w[0].at)
        .map(|w| (w[1].pos - w[0].pos).abs())
        .sum::<f64>();

    // Convert percent/ms -> BPM:
    // raw_percent_per_ms = raw_sum_percent / window_duration_ms
    // BPM = raw_percent_per_ms * 300.0  (derivation: 200% per full thrust, 60s/min => factor 1000*60/200 = 300)
    let raw_percent_per_ms = raw_sum_percent / window_duration_ms as f64;
    let bpm = raw_percent_per_ms * 300.0;

    let intensity = map_bpm_to_intensity(bpm);
    if intensity.is_finite() {
        intensity
    } else {
        0.0
    }
}
