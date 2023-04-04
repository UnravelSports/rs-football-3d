use bevy::prelude::*;
// crate for handling JSON
use serde_derive::{Deserialize, Serialize};

pub const SOURCE_FPS: f32 = 10.0;

// To use the scructs and their variables outside this file,
// all of them should be set to 'pub'
#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerFrame {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub pid: u32,
    pub team: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BallFrame {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MatchFrames {
    pub players: Vec<Vec<PlayerFrame>>,
    pub ball: Vec<BallFrame>,
}

#[derive(Resource)]
pub struct MatchData {
    pub data: MatchFrames,
    pub t: f32,
}

impl MatchData {
    pub fn get_interpolation_values_and_increment(&mut self, delta_time: f32) -> (usize, f32) {
        /*!
        We assume fps is 10. So for every 0.1 second we get a new frame
           if self.t = 0.08 we need to get (x[n] * (1-(self.t-FLOOR(self.t)) + x[n+1] * (self.t-FLOOR(self.t)))/2 to interpolate location
           here n is an index associated to the FLOOR(self.t * fps)
         */
        let mut alpha: f32 = self.t - self.t.floor(); // (self.t-FLOOR(self.t), always a value between 0 and 1
        let mut idx: usize = (self.t * SOURCE_FPS).floor() as usize; // n

        // we do minus one because we can't index length+1 to interpolate from, because that value doesn't exist
        if idx >= self.data.players.len() - 1 {
            self.t = 0.0;
            idx = 0;
            alpha = 0.0;
        } else {
            self.t += delta_time;
        }
        (idx, alpha)
    }
}

pub fn interpolate(alpha: f32, v1: f32, v2: f32) -> f32 {
    (v1 * (1.0 - alpha)) + (v2 * alpha)
}