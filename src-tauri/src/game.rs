use const_for::const_for;
use rand;

pub const fn transpose(state: u64) -> u64 {
    let a1: u64 = state & 0xF0F0_0F0F_F0F0_0F0F;
    let a2: u64 = state & 0x0000_F0F0_0000_F0F0;
    let a3: u64 = state & 0x0F0F_0000_0F0F_0000;
    let a: u64 = a1 | (a2 << 12) | (a3 >> 12);
    let b1: u64 = a & 0xFF00_FF00_00FF_00FF;
    let b2: u64 = a & 0x00FF_00FF_0000_0000;
    let b3: u64 = a & 0x0000_0000_FF00_FF00;
    b1 | (b2 >> 24) | (b3 << 24)
}

const fn reverse_row(row: u64) -> u64 {
    ((row & 0xF) << 12) | ((row & 0xF0) << 4) | ((row & 0xF00) >> 4) | ((row & 0xF000) >> 12)
}

const fn flatten(row: u64) -> u64 {
    let mut i = 0;
    let mut row = row;
    let mut old_row = row;
    while i < 3 {
        let cell = row & (0xF << i * 4);
        if cell == 0 {
            // skip the empty cell
            let first_bits = row & ((1 << (i * 4)) - 1);
            let new_row = (row & !first_bits) >> 4;
            row = new_row | first_bits;
        }
        if row == old_row {
            i += 1;
        } else {
            old_row = row;
        }
    }
    row
}

const fn combine(row: u64) -> (u64, u64) {
    let mut row = flatten(row);
    let mut score = 0u64;

    const_for!(i in 0..3 => {
        let cell = (row >> (i * 4)) & 0xF;
        let next_cell = (row >> ((i + 1) * 4)) & 0xF;
        if cell != 0 && cell == next_cell {
            if cell != 0xF {
                row += 1 << (i * 4);
            }
            row &= !(0xF << ((i + 1) * 4));
            score += 1u64 << (cell + 1);
        }
    });

    (flatten(row), score)
}

const fn row_move_left(row: u64) -> u64 {
    let (new_row, _score) = combine(row);
    new_row
}

const fn game_score_row(row: u64) -> u64 {
    let (_new_row, score) = combine(row);
    score
}

const fn make_move_left_table() -> [u64; 65536] {
    let mut table = [0u64; 65536];
    const_for!(row in 0..65536 => {
        table[row] = (row as u64) ^ row_move_left(row as u64);
    });
    table
}

const fn make_move_right_table() -> [u64; 65536] {
    let mut table = [0u64; 65536];
    const_for!(row in 0..65536 => {
        table[row] = (row as u64) ^ reverse_row(row_move_left(reverse_row(row as u64)));
    });
    table
}

const fn make_move_up_table() -> [u64; 65536] {
    let mut table = [0u64; 65536];
    const_for!(row in 0..65536 => {
        table[row] = transpose(row as u64) ^ transpose(row_move_left(row as u64));
    });
    table
}

const fn make_move_down_table() -> [u64; 65536] {
    let mut table = [0u64; 65536];
    const_for!(row in 0..65536 => {
        table[row] = transpose(row as u64) ^ transpose(reverse_row(row_move_left(reverse_row(row as u64))));
    });
    table
}

const fn make_game_score_table() -> [u64; 65536] {
    let mut table = [0u64; 65536];
    const_for!(row in 0..65536 => {
        table[row] = game_score_row(row as u64);
    });
    table
}

static MOVE_LEFT_TABLE: [u64; 65536] = make_move_left_table();
static MOVE_RIGHT_TABLE: [u64; 65536] = make_move_right_table();
static MOVE_UP_TABLE: [u64; 65536] = make_move_up_table();
static MOVE_DOWN_TABLE: [u64; 65536] = make_move_down_table();
static GAME_SCORE_TABLE: [u64; 65536] = make_game_score_table();

pub const fn game_score_vertical(state: u64) -> u64 {
    let state_t = transpose(state);
    let row0 = (state_t & 0xFFFF) as usize;
    let row1 = ((state_t >> 16) & 0xFFFF) as usize;
    let row2 = ((state_t >> 32) & 0xFFFF) as usize;
    let row3 = ((state_t >> 48) & 0xFFFF) as usize;

    GAME_SCORE_TABLE[row0]
        + GAME_SCORE_TABLE[row1]
        + GAME_SCORE_TABLE[row2]
        + GAME_SCORE_TABLE[row3]
}

pub const fn game_score_horizontal(state: u64) -> u64 {
    let row0 = (state & 0xFFFF) as usize;
    let row1 = ((state >> 16) & 0xFFFF) as usize;
    let row2 = ((state >> 32) & 0xFFFF) as usize;
    let row3 = ((state >> 48) & 0xFFFF) as usize;

    GAME_SCORE_TABLE[row0]
        + GAME_SCORE_TABLE[row1]
        + GAME_SCORE_TABLE[row2]
        + GAME_SCORE_TABLE[row3]
}

pub const fn move_left(state: u64) -> u64 {
    let row0 = (state & 0xFFFF) as usize;
    let row1 = ((state >> 16) & 0xFFFF) as usize;
    let row2 = ((state >> 32) & 0xFFFF) as usize;
    let row3 = ((state >> 48) & 0xFFFF) as usize;

    state
        ^ (MOVE_LEFT_TABLE[row0]
            | (MOVE_LEFT_TABLE[row1] << 16)
            | (MOVE_LEFT_TABLE[row2] << 32)
            | (MOVE_LEFT_TABLE[row3] << 48))
}

pub const fn move_right(state: u64) -> u64 {
    let row0 = (state & 0xFFFF) as usize;
    let row1 = ((state >> 16) & 0xFFFF) as usize;
    let row2 = ((state >> 32) & 0xFFFF) as usize;
    let row3 = ((state >> 48) & 0xFFFF) as usize;

    state
        ^ (MOVE_RIGHT_TABLE[row0]
            | (MOVE_RIGHT_TABLE[row1] << 16)
            | (MOVE_RIGHT_TABLE[row2] << 32)
            | (MOVE_RIGHT_TABLE[row3] << 48))
}

pub const fn move_up(state: u64) -> u64 {
    let state_t = transpose(state);
    let row0 = (state_t & 0xFFFF) as usize;
    let row1 = ((state_t >> 16) & 0xFFFF) as usize;
    let row2 = ((state_t >> 32) & 0xFFFF) as usize;
    let row3 = ((state_t >> 48) & 0xFFFF) as usize;

    state
        ^ (MOVE_UP_TABLE[row0]
            | (MOVE_UP_TABLE[row1] << 4)
            | (MOVE_UP_TABLE[row2] << 8)
            | (MOVE_UP_TABLE[row3] << 12))
}

pub const fn move_down(state: u64) -> u64 {
    let state_t = transpose(state);
    let row0 = (state_t & 0xFFFF) as usize;
    let row1 = ((state_t >> 16) & 0xFFFF) as usize;
    let row2 = ((state_t >> 32) & 0xFFFF) as usize;
    let row3 = ((state_t >> 48) & 0xFFFF) as usize;

    state
        ^ (MOVE_DOWN_TABLE[row0]
            | (MOVE_DOWN_TABLE[row1] << 4)
            | (MOVE_DOWN_TABLE[row2] << 8)
            | (MOVE_DOWN_TABLE[row3] << 12))
}

pub const fn count_empty(state: u64) -> u32 {
    let mut res = state;
    res |= res >> 2;
    res |= res >> 1;
    res = !res & 0x1111_1111_1111_1111;

    res += res >> 32;
    res += res >> 16;
    res += res >> 8;
    res += res >> 4;
    (res & 0xFu64) as u32
}

pub fn count_distinct_tiles(state: u64) -> u8 {
    let mut bitset = 0u32;
    let mut state = state;
    while state != 0 {
        bitset |= 1 << (state & 0xF);
        state >>= 4;
    }
    bitset >>= 1;
    bitset.count_ones() as u8
}

pub fn spawn_new_random_tile(state: u64) -> u64 {
    let empty_count = count_empty(state);
    if empty_count == 0 {
        return state;
    }
    let mut tile: u64 = if rand::random::<f32>() < 0.9 { 1 } else { 2 };
    let mut idx = rand::random::<u32>() % empty_count;
    let mut tmp = state;
    loop {
        while (tmp & 0xF) != 0 {
            tmp >>= 4;
            tile <<= 4;
        }
        if idx == 0 {
            break;
        }
        idx -= 1;
        tmp >>= 4;
        tile <<= 4;
    }
    state | tile
}
