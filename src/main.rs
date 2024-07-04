use std::fmt::Display;

use rand::{seq::SliceRandom, thread_rng};

#[derive(Clone, Copy, PartialEq, Eq)]
/// point, kind
struct Unit(u8, u8);

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self == &Unit(0, 0) {
            return write!(f, " ");
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

struct State(Vec<Unit>);

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

        Self(units)
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

    fn find_free(&self) -> Vec<Unit> {
        let mut free = vec![];
        for (idx, unit) in self.0.iter().enumerate() {
            if unit == &Unit(0, 0) {
                continue;
            }

            if self.is_unit_free(idx as u8) {
                free.push(self.0[idx]);
            }
        }
        free
    }
}

fn main() {
    let state = State::init();
    println!("{}", state);
    println!("{:?}", state.find_free());
}
