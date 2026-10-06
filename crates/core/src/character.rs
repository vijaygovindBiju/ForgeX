use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterProfile {
    pub consistency: u32,
    pub reliability: u32,
    pub resilience: u32,
    pub self_control: u32,
    pub discipline: u32,
    pub overall_level: u32,
}

impl Default for CharacterProfile {
    fn default() -> Self {
        Self {
            consistency: 50,
            reliability: 50,
            resilience: 50,
            self_control: 50,
            discipline: 50,
            overall_level: 1,
        }
    }
}

impl CharacterProfile {
    pub fn new(
        consistency: u32,
        reliability: u32,
        resilience: u32,
        self_control: u32,
        discipline: u32,
    ) -> Self {
        let avg = (consistency + reliability + resilience + self_control + discipline) / 5;
        let overall_level = (avg / 10).max(1);
        Self {
            consistency: consistency.min(100),
            reliability: reliability.min(100),
            resilience: resilience.min(100),
            self_control: self_control.min(100),
            discipline: discipline.min(100),
            overall_level,
        }
    }
}
