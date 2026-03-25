use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Government {
    pub current_ideology: f32, // -1.0 = far left, 1.0 = far right
    pub term_remaining: u32,
}

impl Government {
    pub fn new(ideology: f32, term_length: u32) -> Self {
        Self {
            current_ideology: ideology.clamp(-1.0, 1.0),
            term_remaining: term_length,
        }
    }

    pub fn update_term(&mut self) {
        if self.term_remaining > 0 {
            self.term_remaining -= 1;
        }
    }

    pub fn hold_election(&mut self, citizen_ideologies: &[f32], rng: &mut impl rand::Rng) -> f32 {
        // Simulate imperfect elections with turnout and noise
        
        // Turnout simulation: not all citizens vote
        let turnout_rate = 0.6 + rng.gen::<f32>() * 0.3; // 60-90% turnout
        let voter_count = (citizen_ideologies.len() as f32 * turnout_rate) as usize;
        let voter_count = voter_count.max(1).min(citizen_ideologies.len());
        
        // Randomly select voters
        let mut voter_indices: Vec<usize> = (0..citizen_ideologies.len()).collect();
        for i in 0..voter_indices.len() {
            let j = rng.gen_range(i..voter_indices.len());
            voter_indices.swap(i, j);
        }
        voter_indices.truncate(voter_count);
        
        // Collect votes with some noise and bias
        let mut votes = Vec::with_capacity(voter_count);
        for &idx in &voter_indices {
            let base_vote = citizen_ideologies[idx];
            
            // Add individual noise - increased from ±0.1 to ±0.15 for more imperfection
            let noise = rng.gen_range(-0.15..0.15);
            
            // Bias based on extremeness (extremists more likely to vote)
            let extremeness_bonus = base_vote.abs() * 0.05;
            
            let vote = base_vote + noise + extremeness_bonus;
            votes.push(vote.clamp(-1.0, 1.0));
        }
        
        // Calculate result with some systemic noise
        votes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let new_ideology = if votes.is_empty() {
            0.0
        } else if votes.len() % 2 == 1 {
            votes[votes.len() / 2]
        } else {
            (votes[votes.len() / 2 - 1] + votes[votes.len() / 2]) / 2.0
        };
        
        // Add final systemic noise (media influence, etc.) - increased from ±0.05 to ±0.1
        let systemic_noise = rng.gen_range(-0.1..0.1);
        let final_ideology = (new_ideology + systemic_noise).clamp(-1.0, 1.0);
        
        self.current_ideology = final_ideology;
        self.term_remaining = 50; // Reset term to 50 ticks
        
        final_ideology
    }

    pub fn is_election_due(&self) -> bool {
        self.term_remaining == 0
    }
}
