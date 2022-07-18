use core::ops::Index;
use core::ops::IndexMut;
use fastrand::Rng;
use std::fmt;

#[derive(Default)]
struct ColCounts {
    counts: [u8; 5],
}
#[derive(Debug, Clone, Copy)]
struct ColoredDice {
    col: DieCol,
    count: u8,
}
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum DieCol {
    Red,
    Blue,
    Black,
    Yellow,
    Green,
}
#[derive(Debug, Copy, Clone)]
struct HleConfig {
    my_dice_count: u8,
    their_dice_count: u8,
    rounds: u8,
}
/////////////////////////////

impl fmt::Debug for ColCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entry_iter = DieCol::domain().map(|col| (col, self[col]));
        f.debug_map().entries(entry_iter).finish()
    }
}

impl Index<DieCol> for ColCounts {
    type Output = u8;
    fn index(&self, col: DieCol) -> &u8 {
        &self.counts[col.to_idx() as usize]
    }
}
impl IndexMut<DieCol> for ColCounts {
    fn index_mut(&mut self, col: DieCol) -> &mut u8 {
        &mut self.counts[col.to_idx() as usize]
    }
}
impl ColCounts {
    fn chosen_match(&self, other: &Self) -> Option<ColoredDice> {
        let mut best: Option<ColoredDice> = None;
        for col in DieCol::domain() {
            if self[col] > 0 && self[col] == other[col] {
                match best {
                    Some(colored_dice) if colored_dice.count < self[col] => {}
                    _ => {
                        best = Some(ColoredDice {
                            col,
                            count: self[col],
                        })
                    }
                }
            }
        }
        best
    }
    fn count(&self) -> u8 {
        self.counts.into_iter().sum()
    }
    fn col_rerolled(mut self, col: DieCol, rng: &mut Rng) -> Self {
        let count = self[col];
        self[col] = 0;
        self.with_n_random_more(rng, count)
    }
    fn with_n_random_more(mut self, rng: &mut Rng, count: u8) -> Self {
        for _ in 0..count {
            self[DieCol::random(rng)] += 1;
        }
        self
    }
    fn rerolled(self, rng: &mut Rng) -> Self {
        Self::random(rng, self.count())
    }
    fn random(rng: &mut Rng, count: u8) -> Self {
        Self::default().with_n_random_more(rng, count)
    }
}

impl DieCol {
    fn domain() -> impl Iterator<Item = Self> + Clone {
        (0..5).map(Self::from_idx)
    }
    fn to_idx(self) -> u8 {
        match self {
            Self::Red => 0,
            Self::Blue => 1,
            Self::Black => 2,
            Self::Yellow => 3,
            Self::Green => 4,
        }
    }
    fn from_idx(idx: u8) -> Self {
        // assert!(idx < 5);
        // idx as Self
        match idx {
            0 => Self::Red,
            1 => Self::Blue,
            2 => Self::Black,
            3 => Self::Yellow,
            4 => Self::Green,
            _ => unreachable!(),
        }
    }
    fn random(rng: &mut Rng) -> Self {
        match rng.u8(0..6) {
            0 => Self::Red,
            1 => Self::Blue,
            2 => Self::Black,
            3 => Self::Yellow,
            _ => Self::Green,
        }
    }
}

fn health_lost_experiment(rng: &mut Rng, hle_config: HleConfig) -> u8 {
    let mut mine = ColCounts::random(rng, hle_config.my_dice_count);
    let mut heath_lost: u8 = 0;
    for _ in 0..hle_config.rounds {
        let theirs = ColCounts::random(rng, hle_config.their_dice_count);
        mine = if let Some(colored_dice) = mine.chosen_match(&theirs) {
            heath_lost += colored_dice.count;
            mine.col_rerolled(colored_dice.col, rng)
        } else {
            mine.rerolled(rng)
        };
    }
    heath_lost
}

fn plot_histo(counts_per_symbol: u32, samples: impl Iterator<Item = u8>) {
    let mut buckets = vec![];
    for sample in samples {
        while buckets.len() <= sample as usize {
            buckets.push(0);
        }
        buckets[sample as usize] += 1;
    }
    let count = buckets.iter().copied().sum::<u32>() as f32;
    println!("value | count   | propo. | ascii histogram");
    println!("------+---------+--------+----------------");
    for (i, bucket) in buckets.into_iter().enumerate() {
        print!(
            "{:>5} | {:>7} | {:.4} |",
            i,
            bucket,
            bucket as f32 / count as f32
        );
        for _ in 0..(bucket / counts_per_symbol) {
            print!("#");
        }
        println!();
    }
}

fn main() {
    let mut rng = fastrand::Rng::new();
    let rng = &mut rng;
    for my_dice_count in 3..=8 {
        for their_dice_count in 1..=6 {
            for rounds in 1..=10 {
                let hle_config = HleConfig {
                    my_dice_count,
                    their_dice_count,
                    rounds,
                };
                println!("{:?}", hle_config);
                let samples = (0..100_000).map(|_| health_lost_experiment(rng, hle_config));
                plot_histo(2_000, samples);
                println!();
            }
        }
    }
}
