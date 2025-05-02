//! Computation of maximising expected score for a Farkle game using the Optimal_n iteration algorithm
//! described in the README.md

use crate::hash::{PerfectHash, PerfectHashMap, PerfectHashing};
use crate::farkle::{best_score, best_selection, count_sides, not_busted, score, Dice, DiceSet, DiceSetSample, FarkleScore};
use rayon::prelude::*;
use indicatif::ParallelProgressIterator;
use serde::{Deserialize, Serialize};

/// Manages calculation and storage of results for each calculation of Optimal_n (described in the README.md).
/// new() computes payoffs for Optimal_1 and iterate() computes payoffs for Optimal_n+1
#[derive(Debug, Serialize, Deserialize)]
pub struct OptimalStrat {
    /// Expected score gain values for all possible scores and die subsets
    pub expected_scores: PerfectHashMap<(FarkleScore, [bool; 6]), f32>,
    /// Expected score gain values for all possible scores and die subsets 
    /// assuming the player is definitely going to "Hold" (roll again).
    /// Tuple output of the hashmap stores the expected score gain and the dice that should
    /// be selected to form the hand.
    expected_hold: PerfectHashMap<(FarkleScore, DiceSetSample), (f32, DiceSetSample)>,
    /// The die weightings this strategy is based on
    pub dices: [Dice; 6],
    /// Busting probabilities of the dices
    pub bust_prob: PerfectHashMap<[bool; 6], f32>,
    /// Number of rolls until the "Terminate" strategy must be used
    pub n: usize,
}
impl OptimalStrat {
    /// Computes the expected score for the Optimal_1 strategy with the given die
    pub fn new(dices: [Dice; 6]) -> Self {
        let bust_prob = Self::generate_busting_probabilities(&dices);
        // Since we computing Optimal_1, the "hold" decision is not applicable so we can skip computing it...

        let mut expected_scores = PerfectHashMap::<(FarkleScore, [bool; 6]), f32>::new();
        // For all possible score and dice subset product combinations
        expected_scores.iter_mut()
        .par_bridge()
        .progress_count(<(FarkleScore, [bool; 6])>::SET_SIZE as u64)
        .for_each(|((p, selection), dataslot)| {
            // Calculate expected loss from busting
            let expected_bust_loss = p.score() as f32 * bust_prob[selection];
            // Calculate expected gain when not busting
            let mut expected_score_gain = 0.0;
            let diceset = DiceSet::new(&dices, selection);
            for (sample_wrapped, prob) in diceset.iter_outcomes() {
                expected_score_gain += prob * best_score(count_sides(&sample_wrapped.present())).score() as f32;
            }
            // Store net expected gain
            *dataslot = expected_score_gain - expected_bust_loss;
        });
        // Having no dice means the player loops back round to 6-dice. Thus for any farkle score P,
        // it should be that Optimal(P, no_dice)=Optimal(P, dice)
        for p in (0..FarkleScore::SET_SIZE).map(|h| FarkleScore::from_perfhash(PerfectHash::new(h))) {
            expected_scores[(p, [false; 6])] = expected_scores[(p, [true; 6])];
        }
        return Self {expected_scores, expected_hold: PerfectHashMap::new(), dices, bust_prob, n: 1};
    }
    
    /// Returns the expected score of this strategy with the given current score and boolean mask of dice left
    pub fn query_score(&self, score: FarkleScore, die: [bool; 6]) -> f32 {
        return self.expected_scores[(score, die)];
    }

    /// Returns the decision used by this strategy with the given current score and dice sample.
    /// 
    /// The DiceSetSample returned shows what dice have been selected.
    /// 
    /// The boolean returned indicates whether to roll again.
    ///  - 0 = End turn here
    ///  - 1 = Roll again
    pub fn query_decision(&self, score: FarkleScore, sample: DiceSetSample) -> (DiceSetSample, bool) {
        // Calculate expected loss from busting
        let expected_bust_loss = score.score() as f32 * self.bust_prob[sample.present_mask()];
        // Calculate payoffs
        let terminate = best_score(count_sides(&sample.present())).score() as f32;
        let (hold, hold_selection) = self.expected_hold[(score, sample.clone())].clone();
        if terminate > hold {
            return (best_selection(sample), false);
        }
        return (hold_selection, true);
    }

    /// Computes the expected score for the Optimal_n+1 strategy
    pub fn iterate(&self) -> Self {
        let mut expected_scores = PerfectHashMap::<(FarkleScore, [bool; 6]), f32>::new();
        let expected_hold = self.iterate_hold(&self.dices);
        // For all possible score and dice subset product combinations
        expected_scores.iter_mut()
        .par_bridge()
        .progress_count(<(FarkleScore, [bool; 6])>::SET_SIZE as u64)
        .for_each(|((p, selection), dataslot)| {
            // Calculate expected loss from busting
            let expected_bust_loss = p.score() as f32 * self.bust_prob[selection];
            // Calculate expected gain when not busting
            let mut expected_score_gain = 0.0;
            let diceset = DiceSet::new(&self.dices, selection);
            for (sample_wrapped, prob) in diceset.iter_outcomes() {
                // Calculate terminate decision payoff
                let sample = sample_wrapped.sample.iter().filter_map(|&o| o).collect::<Vec<_>>();
                let terminate = best_score(count_sides(&sample)).score() as f32;
                // Calculate hold decision payoff
                let (hold, _) = expected_hold[(p, sample_wrapped)];
                // Calculate higher of a and b and update expectated score
                expected_score_gain += prob * terminate.max(hold);
            }
            *dataslot = expected_score_gain - expected_bust_loss;
        });
        // Having no dice means the player loops back round to 6-dice. Thus for any farkle score P,
        // it should be that Optimal(P, no_dice)=Optimal(P, dice)
        for p in (0..FarkleScore::SET_SIZE).map(|h| FarkleScore::from_perfhash(PerfectHash::new(h))) {
            expected_scores[(p, [false; 6])] = expected_scores[(p, [true; 6])];
        }
        return Self {expected_scores, expected_hold, dices: self.dices.clone(), bust_prob: self.bust_prob.clone(), n: self.n + 1};
    }

    /// Computes the expected payoff for the "Hold" decision for Optimal_n+1
    fn iterate_hold(&self, dices: &[Dice; 6]) -> PerfectHashMap<(FarkleScore, DiceSetSample), (f32, DiceSetSample)> {
        let mut hold: PerfectHashMap<(FarkleScore, DiceSetSample), (f32, DiceSetSample)> = PerfectHashMap::new();
        hold.iter_mut()
        .par_bridge()
        .progress_count(<(FarkleScore, DiceSetSample)>::SET_SIZE as u64)
        // For all possible scores and dice samples
        .for_each(|((current_score, sample_wrapped), (expected_gain, selection))| {
            let mut best_gain: f32 = 0.0;
            let mut best_selection = DiceSetSample::default();
            // For all possible selections of a sample
            for selection in sample_wrapped.iter_selections() {
                // Calculate score of selection
                let select_score = score(count_sides(&selection.present())).score();
                // Skip any selections that form invalid hands
                if select_score == 0 {
                    continue;
                }
                let selected_dice = DiceSet::new(dices, selection.present_mask());
                let unselected_dice = selected_dice.complement();
                // Calculate "terminate" decision payoff
                let optimal_score = self.expected_scores[(
                    FarkleScore::new((current_score.score() + select_score).clamp(0, 5950)),
                    unselected_dice.select_mask
                )];
                // Store the highest payoff so far
                let total = select_score as f32 + optimal_score;
                if total > best_gain {
                    best_gain = total;
                    best_selection = selection;
                }
            }
            *expected_gain = best_gain;
            *selection = best_selection;
        });
        return hold;
    }

    /// Calculates Farkle busting probabilities for 6 given die
    fn generate_busting_probabilities(dices: &[Dice; 6]) -> PerfectHashMap<[bool; 6], f32> {
        let mut data = PerfectHashMap::new();
        let entire_set = DiceSet::new(dices, [true; 6]);
        // For each possible dice subset
        for subset in entire_set.iter_subsets() {
            // Calculate probability of busting
            let mut bust_prob = 0.0;
            for (sample_wrapped, prob) in subset.iter_outcomes() {
                if !not_busted(&count_sides(&sample_wrapped.present())) {
                    bust_prob += prob;
                }
            }
            
            data[subset.select_mask] = bust_prob;
        }
        // Having no dice means the player loops back round to 6-dice...
        data[[false; 6]] = data[[true; 6]];
        return data;
    }
}
