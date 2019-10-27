//! Motion.

use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use crate::core::Model;
use crate::error::CubismResult;
use crate::json::motion::{Motion3, Segment, SegmentPoint};

fn lerp_points(p0: SegmentPoint, p1: SegmentPoint, t: f32) -> SegmentPoint {
    SegmentPoint {
        time: (p1.time - p0.time).mul_add(t, p0.time),
        value: (p1.value - p0.value).mul_add(t, p0.value),
    }
}

fn segment_intersects(seg: &Segment, t: f32) -> bool {
    match seg {
        Segment::Linear(p0, p1) => p0.time <= t && t <= p1.time,
        Segment::Bezier([p0, _, _, p1]) => p0.time <= t && t <= p1.time,
        Segment::Stepped(p0, t1) => p0.time <= t && t <= *t1,
        Segment::InverseStepped(t0, p1) => *t0 <= t && t <= p1.time,
    }
}

fn segment_interpolate(seg: &Segment, t: f32) -> f32 {
    match seg {
        Segment::Linear(p0, p1) => {
            let k = (t - p0.time) / (p1.time - p0.time);

            if k > 0.0 {
                (p1.value - p0.value).mul_add(k, p0.value)
            } else {
                p0.value
            }
        },
        Segment::Bezier([p0, p1, p2, p3]) => {
            let k = (t - p0.time) / (p3.time - p0.time);
            let k = if k < 0.0 { 0.0 } else { k };

            let (p0, p1, p2, p3) = (*p0, *p1, *p2, *p3);

            let p01 = lerp_points(p0, p1, k);
            let p12 = lerp_points(p1, p2, k);
            let p23 = lerp_points(p2, p3, k);

            let p012 = lerp_points(p01, p12, k);
            let p123 = lerp_points(p12, p23, k);

            lerp_points(p012, p123, k).value
        },
        Segment::Stepped(p0, _) => p0.value,
        Segment::InverseStepped(_, p1) => p1.value,
    }
}

/// Handles motions and animates a model.
#[derive(Clone, Debug)]
pub struct Motion {
    json: Motion3,
    duration: f32,
    fps: f32,
    looped: bool,
    playing: bool,
    current_time: f64,
}

impl Motion {
    /// Creates a Motion from a Motion3.
    pub fn new(motion3: Motion3) -> Motion {
        let duration = motion3.meta.duration;
        let fps = motion3.meta.fps;
        let looped = motion3.meta.looped;

        Motion {
            json: motion3,
            duration,
            fps,
            looped,
            playing: false,
            current_time: 0.0,
        }
    }
    /// Set whether the motion loops.
    pub fn set_looped(&mut self, looped: bool) {
        self.looped = looped;
    }

    /// Plays a motion.
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Pauses a motion.
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Stops a motion.
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_time = 0.0;
    }

    /// Return if the motion playing.
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Creates a Motion from a path of .motion3.json file.
    pub fn from_motion3_json<P: AsRef<Path>>(path: P) -> CubismResult<Motion> {
        let json = Motion3::from_reader(fs::File::open(path)?)?;

        Ok(Motion::new(json))
    }

    /// Ticks frames.
    pub fn tick(&mut self, delta_time: f64) {
        use std::f64;

        if !self.playing {
            return;
        }

        let duration = f64::from(self.duration);

        self.current_time += delta_time;

        if duration <= self.current_time {
            if self.looped {
                self.current_time -= (self.current_time / duration).floor() * duration;
            } else {
                self.current_time = duration;
                self.playing = false;
            }
        }
    }

    /// Updates a model.
    pub fn update(&self, model: &mut Model) -> CubismResult<()> {
        let current = self.current_time as f32;

        let mut lip_sync: Option<f32> = None;
        let mut eye_blink: Option<f32> = None;

        for curve in &self.json.curves {
            for seg in &curve.segments {
                if !segment_intersects(seg, current) {
                    continue;
                }

                let id: &str = &curve.id;
                let target: &str = &curve.target;
                let value = segment_interpolate(seg, current);

                match target {
                    "Model" => {
                        match id {
                            "EyeBlink" => {
                                eye_blink = Some(value);
                            },
                            "LipSync" => {
                                lip_sync = Some(value);
                            },
                            "Opacity" => {
                                // TODO:
                            },
                            _ => {
                                eprintln!("Unhandled id: {}", id);
                            },
                        }
                    },
                    "PartOpacity" => {
                        let param = model.part_mut(id);
                        if let Some(param) = param {
                            *param.opacity = value;
                        }
                    },
                    "Parameter" => {
                        let param = model.parameter_mut(id);
                        if let Some(param) = param {
                            // TODO: fade-in capability
                            *param.value = value;

                            if let Some(_value) = eye_blink {
                                // TODO: multiply eye_blink to value if the
                                // parameter corresponds to
                                // eye blinking
                            }

                            if let Some(_value) = lip_sync {
                                // TODO: add eye_blink to value if the parameter
                                // corresponds to
                                // lip-sync
                            }
                        }
                    },
                    _ => {
                        eprintln!("Unhandled target: {}", target);
                    },
                }

                break;
            }
        }

        if eye_blink.is_none() {
            // TODO: handle eye blinking when not overwritten
        }

        if lip_sync.is_none() {
            // TODO: handle lip syncing when not overwritten
        }

        // TODO: Better error handling
        Ok(())
    }
}

impl From<Motion3> for Motion {
    fn from(motion: Motion3) -> Self {
        Self::new(motion)
    }
}

impl Deref for Motion {
    type Target = Motion3;
    fn deref(&self) -> &Self::Target {
        &self.json
    }
}

impl DerefMut for Motion {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.json
    }
}
