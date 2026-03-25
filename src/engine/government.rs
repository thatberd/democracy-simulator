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

    pub fn hold_election(&mut self, citizen_ideologies: &[f32]) -> f32 {
        // Calculate the winning ideology based on citizen votes
        // Citizens vote for the candidate closest to their ideology
        // For simplicity, we use the median of citizen ideologies as the result
        
        let mut ideologies = citizen_ideologies.to_vec();
        ideologies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let new_ideology = if ideologies.is_empty() {
            0.0
        } else if ideologies.len() % 2 == 1 {
            ideologies[ideologies.len() / 2]
        } else {
            (ideologies[ideologies.len() / 2 - 1] + ideologies[ideologies.len() / 2]) / 2.0
        };
        
        self.current_ideology = new_ideology.clamp(-1.0, 1.0);
        self.term_remaining = 50; // Reset term to 50 ticks
        
        new_ideology
    }

    pub fn is_election_due(&self) -> bool {
        self.term_remaining == 0
    }
}
