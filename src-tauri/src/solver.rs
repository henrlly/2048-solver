use const_for::const_for;

use super::game;

use rayon::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// Per-thread transposition table. Rayon's worker threads are long-lived, so each
// worker keeps its cache warm across `find_best_move` calls without any locking
// in the hot path.
thread_local! {
    static CACHE: RefCell<HashMap<u64, (f64, u8)>> = RefCell::new(HashMap::new());
}

#[inline]
fn cache_get(state: u64) -> Option<(f64, u8)> {
    CACHE.with(|c| c.borrow().get(&state).copied())
}

#[inline]
fn cache_put(state: u64, score: f64, depth: u8) {
    CACHE.with(|c| {
        c.borrow_mut().insert(state, (score, depth));
    });
}

static DEPTH_MIN: u8 = 3;
static DEPTH_MAX: u8 = 6;
static DEPTH_DISCOUNT: u8 = 5;

// Chance nodes at this depth or shallower expand their children in parallel;
// deeper nodes recurse serially so task granularity stays coarse.
static PARALLEL_DEPTH: u8 = 0;

pub static SCORE_MONOTONE_POWER: i32 = 4; // must be odd to keep sign after raising to power
pub static SCORE_MONOTONE_WEIGHT: i32 = 47;
pub static SCORE_EMPTY_WEIGHT: i32 = 270;
pub static SCORE_MERGE_WEIGHT: i32 = 700;
pub static SCORE_SUM_WEIGHT: i32 = 11;
pub static SCORE_SUM_POWER: u32 = 3;
pub static SCORE_LOSS_PENALTY: u64 = 200000;

#[allow(long_running_const_eval)]
pub static SCORE_TABLE: [i64; 65536] = make_score_table();

#[allow(long_running_const_eval)]
const fn make_score_table() -> [i64; 65536] {
    let mut table = [0i64; 65536];
    const_for!(row in 0u32..65536u32 => {
        let nums: [u32; 4] = [
            (row >> 0) & 0xF,
            (row >> 4) & 0xF,
            (row >> 8) & 0xF,
            (row >> 12) & 0xF,
        ];

        let mut merges = 0;
        let mut sum = 0;
        let mut empty = 0;
        let mut prev = 0;
        let mut counter = 0;

        const_for!(i in 0..4 => {
            let rank = nums[i];
            sum += rank.pow(SCORE_SUM_POWER);
            if rank == 0 {
                empty += 1;
            } else {
                if rank == prev {
                    counter += 1;
                } else if counter > 0 {
                    merges += counter + 1;
                    counter = 0;
                }
                prev = rank;
            }
        });

        if counter > 0 {
            merges += counter + 1;
        }

        let mut monotone_left: i32 = 0;
        let mut monotone_right: i32 = 0;

        const_for!(i in 1..4 =>  {
            let c_curr: i32 = nums[i] as i32;
            let c_prev: i32 = nums[i - 1] as i32;
            if c_prev > c_curr {
                monotone_left += c_prev.pow(SCORE_MONOTONE_POWER as u32);
                monotone_left -= c_curr.pow(SCORE_MONOTONE_POWER as u32);
            } else {
                monotone_right += c_curr.pow(SCORE_MONOTONE_POWER as u32);
                monotone_right -= c_prev.pow(SCORE_MONOTONE_POWER as u32);
            }
        });

        table[row as usize] = if monotone_left < monotone_right {
            -(monotone_left as i64) * (SCORE_MONOTONE_WEIGHT as i64)
        } else {
            -(monotone_right as i64) * (SCORE_MONOTONE_WEIGHT as i64)
        };
        table[row as usize] += (empty as i64) * (SCORE_EMPTY_WEIGHT as i64);
        table[row as usize] += (merges as i64) * (SCORE_MERGE_WEIGHT as i64);
        table[row as usize] -= (sum as i64) * (SCORE_SUM_WEIGHT as i64);
        table[row as usize] += SCORE_LOSS_PENALTY as i64;
        table[row as usize] *= 100;
    });
    table
}

// scoring position without additional search
const fn score_position(state: u64) -> i64 {
    let row0 = (state & 0xFFFF) as usize;
    let row1 = ((state >> 16) & 0xFFFF) as usize;
    let row2 = ((state >> 32) & 0xFFFF) as usize;
    let row3 = ((state >> 48) & 0xFFFF) as usize;

    let state_transposed = game::transpose(state);
    let col0 = (state_transposed & 0xFFFF) as usize;
    let col1 = ((state_transposed >> 16) & 0xFFFF) as usize;
    let col2 = ((state_transposed >> 32) & 0xFFFF) as usize;
    let col3 = ((state_transposed >> 48) & 0xFFFF) as usize;

    SCORE_TABLE[row0]
        + SCORE_TABLE[row1]
        + SCORE_TABLE[row2]
        + SCORE_TABLE[row3]
        + SCORE_TABLE[col0]
        + SCORE_TABLE[col1]
        + SCORE_TABLE[col2]
        + SCORE_TABLE[col3]
}

pub fn score_post_spawn(state: u64, depth: u8, depth_limit: u8, cprob: f32) -> f64 {
    let left = game::move_left(state);
    let right = game::move_right(state);
    let up = game::move_up(state);
    let down = game::move_down(state);
    if left == state && right == state && up == state && down == state {
        return f64::MIN;
    }

    let left_score = if left != state {
        score_pre_spawn(left, depth + 1, depth_limit, cprob)
    } else {
        f64::MIN
    };
    let right_score = if right != state {
        score_pre_spawn(right, depth + 1, depth_limit, cprob)
    } else {
        f64::MIN
    };
    let up_score = if up != state {
        score_pre_spawn(up, depth + 1, depth_limit, cprob)
    } else {
        f64::MIN
    };
    let down_score = if down != state {
        score_pre_spawn(down, depth + 1, depth_limit, cprob)
    } else {
        f64::MIN
    };

    let res = {
        if left_score >= right_score && left_score >= up_score && left_score >= down_score {
            left_score
        } else if right_score >= left_score && right_score >= up_score && right_score >= down_score
        {
            right_score
        } else if up_score >= left_score && up_score >= right_score && up_score >= down_score {
            up_score
        } else {
            down_score
        }
    };
    res
}

pub fn score_pre_spawn(state: u64, depth: u8, depth_limit: u8, cprob: f32) -> f64 {
    let minprob: f32 = f32::max(0.0001, 1.0 / ((1 << (2 * depth + 4)) as f32));
    if cprob < minprob || depth >= depth_limit {
        if let Some((cached_score, cached_depth)) = cache_get(state) {
            if cached_depth >= depth {
                return cached_score;
            }
        }
        return score_position(state) as f64;
    }

    if let Some((cached_score, cached_depth)) = cache_get(state) {
        if cached_depth >= depth {
            return cached_score;
        }
    }

    let count = game::count_empty(state);
    let cprob = cprob / count as f32;

    // Nibble offsets of the empty cells; one chance-node child per (cell, tile).
    let empty_shifts: Vec<u32> = (0..16u32)
        .filter(|&i| state & (0xF << (i * 4)) == 0)
        .collect();

    let expand = |&shift: &u32| -> f64 {
        let tile: u64 = 1 << (shift * 4);
        score_post_spawn(state | tile, depth, depth_limit, cprob * 0.9) * 0.9
            + score_post_spawn(state | (tile << 1), depth, depth_limit, cprob * 0.1) * 0.1
    };

    let res: f64 = if depth <= PARALLEL_DEPTH {
        empty_shifts.par_iter().map(expand).sum()
    } else {
        empty_shifts.iter().map(expand).sum()
    };

    let result = res / count as f64;
    cache_put(state, result, depth);
    result
}

pub fn find_best_move(state: u64) -> (String, f64) {
    let moves = [
        ("up", game::move_up(state)),
        ("down", game::move_down(state)),
        ("left", game::move_left(state)),
        ("right", game::move_right(state)),
    ];

    let depth_limit = std::cmp::min(
        DEPTH_MAX,
        std::cmp::max(
            DEPTH_MIN as i8,
            game::count_distinct_tiles(state) as i8 - DEPTH_DISCOUNT as i8,
        ) as u8,
    );

    // Score the (up to 4) legal root moves in parallel. `collect` on an indexed
    // parallel iterator preserves order, so the tie-break below is identical to
    // the original serial scan (up > down > left > right).
    let scores: Vec<Option<f64>> = moves
        .par_iter()
        .map(|(_mv, new_state)| {
            if *new_state != state {
                Some(score_pre_spawn(*new_state, 0, depth_limit, 1.0))
            } else {
                None
            }
        })
        .collect();

    let mut best_move: &str = "";
    let mut best_score: f64 = f64::MIN;
    for ((mv, _), score) in moves.iter().zip(scores) {
        if let Some(score) = score {
            if best_score == 0.0 || score > best_score {
                best_score = score;
                best_move = *mv;
            }
        }
    }

    (best_move.to_string(), best_score)
}
