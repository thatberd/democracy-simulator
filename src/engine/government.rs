use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Government {
    pub current_ideology: f32, // -1.0 = far left, 1.0 = far right
    pub term_remaining: u32,
    pub previous_ideology: f32, // For inertia calculations
    pub policy_queue: Vec<f32>, // Queue of past government ideologies for lag effects
}

impl Government {
    pub fn new(ideology: f32, term_length: u32) -> Self {
        let ideology = ideology.clamp(-1.0, 1.0);
        Self {
            current_ideology: ideology,
            term_remaining: term_length,
            previous_ideology: ideology,
            policy_queue: vec![ideology; 15], // Initialize with 15 ticks of current ideology
        }
    }

    pub fn update_term(&mut self) {
        if self.term_remaining > 0 {
            self.term_remaining -= 1;
        }
    }

    pub fn update_policy_queue(&mut self) {
        // Add current ideology to queue and remove oldest
        self.policy_queue.push(self.current_ideology);
        if self.policy_queue.len() > 15 {
            self.policy_queue.remove(0);
        }
    }

    pub fn get_lagged_ideology(&self) -> f32 {
        // Return ideology from 10 ticks ago for economic effects
        if self.policy_queue.len() >= 10 {
            self.policy_queue[self.policy_queue.len() - 10]
        } else {
            self.current_ideology // Fallback if queue not full yet
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
            
            // Add individual noise
            let noise = rng.gen_range(-0.1..0.1);
            
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
        
        // Add final systemic noise (media influence, etc.)
        let systemic_noise = rng.gen_range(-0.05..0.05);
        let new_ideology = (new_ideology + systemic_noise).clamp(-1.0, 1.0);
        
        // Apply inertia to smooth transitions
        let inertia = 0.8;
        let final_ideology = self.current_ideology * inertia + new_ideology * (1.0 - inertia);
        
        self.previous_ideology = self.current_ideology;
        self.current_ideology = final_ideology.clamp(-1.0, 1.0);
        self.term_remaining = 50; // Reset term to 50 ticks
        
        final_ideology
    }

    pub fn is_election_due(&self) -> bool {
        self.term_remaining == 0
    }
}
