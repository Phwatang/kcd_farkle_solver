//! Everything to do with simulating a game of Farkle (using the variant in Kingdom Come Deliverance)

use std::ops::Index;
use std::ops::Deref;

use serde::Deserialize;
use serde::Serialize;

/// The face/outcome of a 6-sided die
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiceSide {
    One = 0, 
    Two = 1, 
    Three = 2, 
    Four = 3, 
    Five = 4, 
    Six = 5,
}
impl From<u8> for DiceSide {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::One,
            1 => Self::Two,
            2 => Self::Three,
            3 => Self::Four,
            4 => Self::Five,
            5 | _ => Self::Six,
        }
    }
}

/// Representation of a fair (or unfair) 6 sided dice
/// 
/// Indexing is implemented for DiceSide (or usize) to retrieve the probability to retrieve that side.
///  - When indexing with usize, example_obj[0] would refer to probability for side 1, example_obj[1] would refer to side 2 etc...
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Dice {
    /// Probabilities for sides 1 to 6 respectively
    probabilities: [f32; 6]
}
impl Dice {
    /// Samples the dice according to its prescribed probabilities
    pub fn roll(&self) -> u8 {
        todo!()
    }
    /// Creates a new Die with specified probabilities. Proability array refers to sides 1 to 6 respectively (in that order).
    pub fn new(probabilities: [f32; 6]) -> Self {
        Self {probabilities}
    }
    /// Creates a new Die with specified probabilities. Weights array refers to the bias for sides 1 to 6 respectively (in that order).
    ///
    /// E.g weightings of [1,2,1,1,1,1] will give the probability distribution of:
    ///  - Side 1: 1/7
    ///  - Side 2: 2/7
    ///  - Side 3: 1/7
    ///  - Side 4: 1/7
    ///  - Side 5: 1/7
    ///  - Side 6: 1/7
    pub fn new_with_weights(weightings: [u32; 6]) -> Self {
        let denominator: f32 = weightings.iter().sum::<u32>() as f32;
        return Self::new(
            weightings.map(|i| i as f32 / denominator)
        );
    }
    /// Calculates the expected outcome for this die
    pub fn expected_roll(&self) -> f32 {
        return (0..6).map(|i| ((i+1) as f32)*self.probabilities[i]).sum();
    }
}
impl Default for Dice {
    /// A fair dice
    fn default() -> Self {
        Self {probabilities: [1.0 / 6.0, 1.0 / 6.0, 1.0 / 6.0, 1.0 / 6.0, 1.0 / 6.0, 1.0 / 6.0]}
    }
}
impl Index<usize> for Dice {
    type Output = f32;

    fn index(&self, ind: usize) -> &Self::Output {
        return &self.probabilities[ind];
    }
}
impl Index<DiceSide> for Dice {
    type Output = f32;

    fn index(&self, ind: DiceSide) -> &Self::Output {
        return &self.probabilities[ind as usize];
    }
}

/// A potential sample result from rolling (up to) 6 die.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DiceSetSample {
    // A sample of (potentially) 6 die. Option::None indicates the dice was missing.
    pub sample: [Option<DiceSide>; 6]
}
impl DiceSetSample {
    pub fn new(sample: [Option<DiceSide>; 6]) -> Self {
        return Self {sample};
    }

    // Return vec of dice sides that are present in the sample
    pub fn present(&self) -> Vec<DiceSide> {
        return self.sample.iter()
            .filter_map(|&o| o)
            .collect();
    }

    pub fn present_mask(&self) -> [bool; 6] {
        return self.sample.map(|o| o.is_some());
    }

    // Iterates through all possible "selections" of this sample. In otherwords, all possible
    // subsets of what is present in this sample, will be mapped to its own sample.
    pub fn iter_selections(&self) -> impl ExactSizeIterator<Item = DiceSetSample> {
        // Get active number of dice
        let n = self.sample.iter().filter(|&o| o.is_some()).count();
        // Iterate through the subsets. This is done by iterating through all possibilities of
        // replacing Option::Some with Option::None
        return (1..(2usize.pow(n as u32))).map(|i| {
            let mut out = self.clone();
            let mut val = i;
            for slot in out.sample.iter_mut().filter(|o| o.is_some()) {
                if (val % 2) != 0 {
                    *slot = None;
                }
                val = val / 2
            }
            return out;
        })
    }
}

#[derive(Clone, Debug)]
/// A set/sub-set of 6 existing die
pub struct DiceSet<'a> {
    // The 6 die this (sub)set is based on
    pub dices: &'a [Dice; 6],
    // Boolean mask for what dice are retained in this set. true = retain, false = exclude.
    pub select_mask: [bool; 6]
}
impl<'a> DiceSet<'a> {
    pub fn new(dices: &'a [Dice; 6], select_mask: [bool; 6]) -> Self {
        Self {dices, select_mask}
    }

    /// Returns the complement set of this set
    pub fn complement(&self) -> Self {
        let mut out = self.clone();
        out.select_mask = out.select_mask.map(|b| !b);
        return out;
    }

    /// Creates an iterator that iterates through all the sampling possibilities of this DiceSet along with their respective probabilities
    pub fn iter_outcomes(&self) -> impl ExactSizeIterator<Item = (DiceSetSample, f32)> {
        // Get active number of dice
        let n = self.select_mask.iter().filter(|&bit| *bit == true).count();
        // Iterate through the sampling combinations of the active dice
        return (0..(6usize.pow(n as u32)))
            .map(move |mut i| {
                let mut prob = 1.0;
                let mut v = DiceSetSample::default();
                for (ind, _) in self.select_mask.iter().enumerate().filter(|&(_, bit)| *bit == true) {
                    let side: DiceSide = ((i % 6) as u8).into();
                    v.sample[ind] = Some(side);
                    prob *= self.dices[ind][side];
                    i = i / 6
                }
                return (v, prob);
            });
    }

    /// Creates an iterator that iterates through all the possible (non-empty) subsets of this DiceSet.
    /// 
    /// Note that these are not strict subsets. I.e a clone of this object will be yielded somewhere along the iteration.
    pub fn iter_subsets(&self) -> impl ExactSizeIterator<Item = DiceSet<'a>> {
        // Get active number of dice
        let n_active = self.select_mask.iter().filter(|&bit| *bit == true).count();
        // Iterate through all possible subsets of the selection mask
        return (1..(2usize.pow(n_active as u32))).map(|i| {
            let mut val = i;
            let mut arr: [bool; 6] = [false; 6];
            for (arr_slot, _) in arr.iter_mut().zip(self.select_mask).filter(|(_, bit)| *bit == true) {
                *arr_slot = (val % 2) != 0;
                val = val / 2
            }
            return Self::new(self.dices, arr);
        })
    }

    /// Creates a new subset from this set using the given (boolean) selection mask.
    ///
    /// Selection mask details:
    ///  - true means retain and false means exclude 
    ///     - E.g [1,0,1] means 1st and 3rd die are in the new subset
    ///  - If the set is longer than the selection mask, any dice not included in the selection mask are excluded
    ///  - If the set is shorter than the selection mask, the selection mask is internally truncated to the length of the set
    pub fn new_subset(&self, select_mask: &[bool; 6]) -> Self {
        let mut mask = [false; 6];
        for ((slot, &bit1), &bit2) in mask.iter_mut().zip(self.select_mask.iter()).zip(select_mask.iter()) {
            *slot = bit1 && bit2;
        }
        return Self::new(self.dices, mask);
    }
}

/// Counts up the amount of times each DiceSide occurs in the slice given
pub fn count_sides(sides: &[DiceSide]) -> [u8; 6] {
    let out = sides.iter().enumerate().fold([0u8; 6], |mut acc, (_, &side)| {
        acc[side as usize] += 1;
        return acc
    });
    return out;
}

/// Helper function to detect triples, quads, pentas etc
fn highest_multi(occurances: &[u8; 6]) -> (u8, u8, u32) {
    let mut best_side: u8 = 0;
    let mut best_count: u8 = 0;
    let mut best_score: u32 = 0;
    // For each side that has >= 3 occurances
    for (side, &count) in occurances.iter().enumerate()
    .filter_map(|(i, j)| {
        if *j >= 3 {
            return Some((i+1, j));
        }
        return None;
    }) {
        let mut score: u32;
        match side {
            1 => score = 1000,
            _ => score = (side as u32) * 100,
        }
        score *= 2u32.pow((count as u32) - 3);
        if score > best_score {
            best_side = side as u8;
            best_count = count;
            best_score = score;
        }
    }
    return (best_side, best_count, best_score);
}

/// Detects if a given dice sample can score >0 points
pub fn not_busted(occurances: &[u8; 6]) -> bool {
    match occurances {
        // Full straight
        [1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX] => {
            return true;
        }
        // Partial straight (starting at 2)
        [_, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX] => {
            return true;
        }
        // Partial straight (starting at 1)
        [1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, _] => {
            return true;
        }
        _ => {},
    }
    match highest_multi(&occurances) {
        (0, ..) => {},
        (_, _, _) => {
            return true;
        }
    }
    if occurances[0] > 0 {
        return true;
    }
    if occurances[4] > 0 {
        return true;
    }
    return false;
}

/// Represents a score within a game of Farkle.
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct FarkleScore {
    // The score
    pub value: u32
}
impl Deref for FarkleScore {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        return &self.value;
    }
}
impl FarkleScore {
    pub fn new(score: u32) -> Self {
        return Self {value: score}
    }
    pub fn score(&self) -> u32 {
        return self.value
    }
}


/// Calculates Farkle score given the results of 6 or less dice.
/// 
/// Occurances array is the number of times each number occured from the set of dice.
///  - E.g We roll 2x 3's, 2x 4's and 1x 6. Occurances would be: [0,0,2,2,1]
///  - If all the occurances are >6, score calculation is not guaranteed to be correct
pub fn score(mut occurances: [u8; 6]) -> FarkleScore {
    let mut output: u32 = 0;
    loop {
        match occurances {
            // Full straight
            [1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX] => {
                output += 1500;
                occurances.iter_mut().for_each(|i| *i -= 1);
                continue;
            }
            // Partial straight (starting at 2)
            [_, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX] => {
                output += 750;
                occurances.iter_mut().skip(1).for_each(|i| *i -= 1);
                continue;
            }
            // Partial straight (starting at 1)
            [1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, _] => {
                output += 500;
                occurances.iter_mut().take(5).for_each(|i| *i -= 1);
                continue;
            }
            _ => {},
        }
        match highest_multi(&occurances) {
            (0, ..) => {},
            (side, count, sc) => {
                output += sc;
                occurances[(side-1) as usize] -= count;
                continue;
            }
        }
        if occurances[0] > 0 {
            occurances[0] -= 1;
            output += 100;
            continue;
        }
        if occurances[4] > 0 {
            occurances[4] -= 1;
            output += 50;
            continue;
        }
        break;
    }
    // If all the dice have not been "used up" for the scoring calculation then we
    // have an invalid hand. Thus score is 0.
    if occurances != [0,0,0,0,0,0] {
        return FarkleScore::new(0);
    }
    return FarkleScore::new(output);
}

/// Calculates the best Farkle score given the results of 6 or less dice.
pub fn best_score(mut occurances: [u8; 6]) -> FarkleScore {
    let mut output: u32 = 0;
    loop {
        match occurances {
            // Full straight
            [1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX] => {
                output += 1500;
                occurances.iter_mut().for_each(|i| *i -= 1);
                continue;
            }
            // Partial straight (starting at 2)
            [_, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX] => {
                output += 750;
                occurances.iter_mut().skip(1).for_each(|i| *i -= 1);
                continue;
            }
            // Partial straight (starting at 1)
            [1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, _] => {
                output += 500;
                occurances.iter_mut().take(5).for_each(|i| *i -= 1);
                continue;
            }
            _ => {},
        }
        match highest_multi(&occurances) {
            (0, ..) => {},
            (side, count, sc) => {
                output += sc;
                occurances[(side-1) as usize] -= count;
                continue;
            }
        }
        if occurances[0] > 0 {
            occurances[0] -= 1;
            output += 100;
            continue;
        }
        if occurances[4] > 0 {
            occurances[4] -= 1;
            output += 50;
            continue;
        }
        break;
    }
    return FarkleScore::new(output);
}

/// Calculates the dice sides to be chosen in the sample to achieve the highest scoring Farkle
/// hand. Can be thought of as the dual to best_score()
pub fn best_selection(sample: DiceSetSample) -> DiceSetSample {
    // Figure out what is "used" when forming the best hand
    let mut occurances = count_sides(&sample.present());
    loop {
        match occurances {
            // Full straight
            [1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX] => {
                occurances.iter_mut().for_each(|i| *i -= 1);
                continue;
            }
            // Partial straight (starting at 2)
            [_, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX] => {
                occurances.iter_mut().skip(1).for_each(|i| *i -= 1);
                continue;
            }
            // Partial straight (starting at 1)
            [1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, 1..=u8::MAX, _] => {
                occurances.iter_mut().take(5).for_each(|i| *i -= 1);
                continue;
            }
            _ => {},
        }
        match highest_multi(&occurances) {
            (0, ..) => {},
            (side, count, sc) => {
                occurances[(side-1) as usize] -= count;
                continue;
            }
        }
        if occurances[0] > 0 {
            occurances[0] -= 1;
            continue;
        }
        if occurances[4] > 0 {
            occurances[4] -= 1;
            continue;
        }
        break;
    }
    // The left over dice afterwards are not need when forming the best hand.
    // So to form the best hand we copy the dice sample and remove the left over dice.
    let mut out = sample.clone();
    for (side, &count) in occurances.iter().enumerate().map(|(i, c)| (DiceSide::from(i as u8), c)) {
        for op in out.sample.iter_mut().filter(|o| **o == Some(side)) {
            *op = None;
            if count == 0 {
                break;
            }
        }
    }
    return out;
}
