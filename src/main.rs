use std::{
    collections::BTreeMap,
    fmt::{Display, Write as _},
    io::{BufRead, Write},
    sync::atomic::AtomicU64,
};

use rand::{seq::SliceRandom, thread_rng};
use rayon::prelude::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// point, kind
struct Unit(u8, u8);

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self == &Unit(0, 0) {
            return write!(f, "{:3}", "\u{1f02b}");
        }
        let unit_repr = [
            ['🀇', '🀈', '🀉', '🀊', '🀋', '🀌', '🀍', '🀎', '🀏', '\u{1f029}'],
            ['🀐', '🀑', '🀒', '🀓', '🀔', '🀕', '🀖', '🀗', '🀘', '🀅'],
            ['🀙', '🀚', '🀛', '🀜', '🀝', '🀞', '🀟', '🀠', '🀡', '🀆'],
        ];
        write!(f, "{:3}", unit_repr[self.1 as usize][(self.0 - 1) as usize])
    }
}

impl std::fmt::Debug for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Copy)]
struct State([Unit; 120]);

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, unit) in self.0.iter().enumerate() {
            if idx % 10 == 0 {
                writeln!(f)?;
            }
            write!(f, "{:3}", unit)?;
        }
        Ok(())
    }
}

impl State {
    // 10 * 3 * 4
    pub fn init() -> Self {
        let mut units = vec![];
        for point in 0..10 {
            for kind in 0..3 {
                units.append(&mut vec![Unit(point + 1, kind); 4]);
            }
        }
        // Shuffle units
        units.shuffle(&mut thread_rng());

        Self(units.try_into().unwrap())
    }

    pub fn is_unit_free(&self, idx: u8) -> bool {
        let row = (idx / 10) as usize;
        let col = (idx % 10) as usize;

        // up
        if row == 0 {
            return true;
        }
        for i in (0..row).rev() {
            if self.0[10 * i + col] != Unit(0, 0) {
                break;
            }

            if i == 0 {
                return true;
            }
        }

        // down
        if row == 11 {
            return true;
        }
        for i in (row + 1)..12 {
            if self.0[10 * i + col] != Unit(0, 0) {
                break;
            }
            if i == 11 {
                return true;
            }
        }

        // left
        if col == 0 {
            return true;
        }
        for i in (0..col).rev() {
            if self.0[10 * row + i] != Unit(0, 0) {
                break;
            }
            if i == 0 {
                return true;
            }
        }

        // right
        if col == 9 {
            return true;
        }
        for i in (col + 1)..10 {
            if self.0[10 * row + i] != Unit(0, 0) {
                break;
            }
            if i == 9 {
                return true;
            }
        }

        false
    }

    fn find_free(&self) -> Vec<usize> {
        let mut free = vec![];
        for (idx, unit) in self.0.iter().enumerate() {
            if unit == &Unit(0, 0) {
                continue;
            }

            if self.is_unit_free(idx as u8) {
                free.push(idx);
            }
        }
        free
    }

    fn find_pair(&self) -> Vec<(usize, usize)> {
        let mut units: BTreeMap<Unit, Vec<usize>> = BTreeMap::new();
        for idx in self.find_free() {
            if let Some(unit) = units.get_mut(&self.0[idx]) {
                unit.push(idx);
            } else {
                units.insert(self.0[idx], vec![idx]);
            }
        }

        let mut pairs = vec![];
        for (_, idxs) in units.into_iter() {
            match idxs.len() {
                2 => pairs.push((idxs[0], idxs[1])),
                3 => {
                    pairs.push((idxs[0], idxs[1]));
                    pairs.push((idxs[0], idxs[2]));
                    pairs.push((idxs[1], idxs[2]));
                }
                4 => {
                    pairs.push((idxs[0], idxs[1]));
                    pairs.push((idxs[0], idxs[2]));
                    pairs.push((idxs[0], idxs[3]));
                    pairs.push((idxs[1], idxs[2]));
                    pairs.push((idxs[1], idxs[3]));
                    pairs.push((idxs[2], idxs[3]));
                }
                _ => {}
            }
        }
        pairs
    }

    fn remove_pair(&mut self, (a, b): (usize, usize)) -> eyre::Result<usize> {
        if self.0[a] != self.0[b] {
            return Err(eyre::eyre!("Invalid pair"));
        }

        let p = self.0[a].0;
        self.0[a] = Unit(0, 0);
        self.0[b] = Unit(0, 0);
        Ok(p as usize)
    }
}

static FINAL_STATE: AtomicU64 = AtomicU64::new(0);

fn step(state: State, point: usize, turns: usize) -> usize {
    let pairs = state.find_pair();
    if pairs.is_empty() || turns > 0 {
        print!(
            "Final: {}\r",
            FINAL_STATE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        return point;
    }

    let t = pairs.into_iter().map(|pair| {
        let mut new_state = state;
        let p = new_state.remove_pair(pair).unwrap();
        step(
            new_state,
            point + if turns % 2 == 0 { p } else { 0 },
            turns + 1,
        )
    });

    if turns % 2 == 0 {
        t.max().unwrap()
    } else {
        t.min().unwrap()
    }
}

fn main() {
    let mut state = State::init();
    let mut player_score = 0;
    let mut bot_score = 0;
    let mut turns = 0;
    loop {
        println!("{}", state);
        loop {
            let mut input = String::new();
            std::io::stdin().lock().read_line(&mut input).unwrap();
            let a = input
                .split(' ')
                .map(|s| s.trim().parse::<usize>().unwrap())
                .collect::<Vec<_>>();
            if let Ok(p) = state.remove_pair((a[0], a[1])) {
                player_score += p;
                break;
            };
        }
    }
}
