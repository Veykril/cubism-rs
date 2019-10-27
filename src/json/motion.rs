//! Parses .motion3.json.
use serde::{self, Deserialize, Serialize};

use std::str::FromStr;

/// Rust structure representation for Motion3 metadata.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Meta {
    /// Duration of a motion.
    pub duration: f32,
    /// Frame per second.
    pub fps: f32,
    #[serde(rename = "Loop")]
    /// True if the motion is looped.
    pub looped: bool,
    /// TODO:
    #[serde(rename = "AreBeziersRestricted")]
    pub restricted_beziers: bool,
    /// A number of curves that the motion3.json file has.
    pub curve_count: usize,
    /// A number of segments that the motion3.json file has.
    pub total_segment_count: usize,
    /// A number of points that the motion3.json file has.
    pub total_point_count: usize,
    /// A number of user data fields that the motion3.json file has.
    pub user_data_count: usize,
    /// A total size of user data.
    pub total_user_data_size: usize,
}

/// Point.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SegmentPoint {
    /// Time.
    pub time: f32,
    /// Value.
    pub value: f32,
}

/// Segment.
#[derive(Copy, Clone, Debug)]
pub enum Segment {
    /// Linear.
    Linear(SegmentPoint, SegmentPoint),
    /// Bezier curve.
    Bezier([SegmentPoint; 4]),
    /// Stepped.
    Stepped(SegmentPoint, f32),
    /// Inverse stepped.
    InverseStepped(f32, SegmentPoint),
}

mod segment_parser {
    use crate::json::motion::{Segment, SegmentPoint};
    use serde::{
        self,
        de::{self, SeqAccess, Visitor},
        Deserializer, Serializer,
    };

    struct SegmentVisitor;

    impl<'de> Visitor<'de> for SegmentVisitor {
        type Value = Vec<Segment>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("sequence of integers / numbers")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
            let mut seq = seq;
            let mut ret = vec![];

            const SEG_LINEAR: i32 = 0; // リニア
            const SEG_BEZIER: i32 = 1; // ベジェ曲線
            const SEG_STEPPED: i32 = 2; // ステップ
            const SEG_INV: i32 = 3; // インバースステップ

            // parse the first position
            let t0: f32 = seq.next_element()?.unwrap();
            let v0: f32 = seq.next_element()?.unwrap();

            let mut last_point = SegmentPoint {
                time: t0,
                value: v0,
            };

            // parse positions
            while let Some(seg_type) = seq.next_element()? as Option<i32> {
                match seg_type {
                    SEG_LINEAR => {
                        let t0: f32 = seq.next_element()?.unwrap();
                        let v0: f32 = seq.next_element()?.unwrap();

                        let next_point = SegmentPoint {
                            time: t0,
                            value: v0,
                        };

                        ret.push(Segment::Linear(last_point, next_point));
                        last_point = next_point;
                    },
                    SEG_STEPPED => {
                        let t0: f32 = seq.next_element()?.unwrap();
                        let v0: f32 = seq.next_element()?.unwrap();

                        ret.push(Segment::Stepped(last_point, t0));

                        last_point = SegmentPoint {
                            time: t0,
                            value: v0,
                        };
                    },
                    SEG_INV => {
                        let t0: f32 = seq.next_element()?.unwrap();
                        let v0: f32 = seq.next_element()?.unwrap();

                        let tn = last_point.time;

                        last_point = SegmentPoint {
                            time: t0,
                            value: v0,
                        };

                        ret.push(Segment::InverseStepped(tn, last_point));
                    },
                    SEG_BEZIER => {
                        let t0: f32 = seq.next_element()?.unwrap();
                        let v0: f32 = seq.next_element()?.unwrap();
                        let t1: f32 = seq.next_element()?.unwrap();
                        let v1: f32 = seq.next_element()?.unwrap();
                        let t2: f32 = seq.next_element()?.unwrap();
                        let v2: f32 = seq.next_element()?.unwrap();

                        let next_point = SegmentPoint {
                            time: t2,
                            value: v2,
                        };

                        ret.push(Segment::Bezier([
                            last_point,
                            SegmentPoint {
                                time: t0,
                                value: v0,
                            },
                            SegmentPoint {
                                time: t1,
                                value: v1,
                            },
                            next_point,
                        ]));

                        last_point = next_point;
                    },
                    _ => return Err(de::Error::custom("invalid segment format.")),
                }
            }

            Ok(ret)
        }
    }

    pub fn serialize<S>(_: &[Segment], _: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unimplemented!()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Segment>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SegmentVisitor)
    }
}

/// Rust structure representation for Motion3 curve data.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Curve {
    /// Target.
    pub target: String,
    /// Id.
    pub id: String,
    /// Segments.
    #[serde(with = "segment_parser")]
    pub segments: Vec<Segment>,
    /// Fade-in time. 1.0 [sec] as default.
    #[serde(default = "super::float_1")]
    pub fade_in_time: f32,
    /// Fade-out time. 1.0 [sec] as default.
    #[serde(default = "super::float_1")]
    pub fade_out_time: f32,
}

/// Rust structure representation for Motion3.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Motion3 {
    /// Version.
    pub version: u32,
    /// Metadata.
    pub meta: Meta,
    /// Curves.
    pub curves: Vec<Curve>,
    #[serde(default)]
    pub user_data: Vec<MotionUserData>,
}

impl Motion3 {
    /// Parses a Motion3 from a .motion3.json reader.
    #[inline]
    pub fn from_reader<R: std::io::Read>(r: R) -> serde_json::Result<Self> {
        serde_json::from_reader(r)
    }
}

impl FromStr for Motion3 {
    type Err = serde_json::Error;

    /// Parses a Motion3 from a .motion3.json string.
    #[inline]
    fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MotionUserData {
    pub time: f32,
    pub value: String,
}

#[test]
fn json_samples_motion3() {
    use std::iter::FromIterator;
    let path = std::path::PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);
    for model in &["Haru", "Hiyori", "Mark", "Natori"] {
        let motion_path = path.join([model, "/motions/"].concat());
        let motions = std::fs::read_dir(motion_path).unwrap();

        for motion in motions {
            let motion_path = motion.unwrap().path();

            if !motion_path.is_file() {
                continue;
            }

            Motion3::from_str(
                &std::fs::read_to_string(&motion_path)
                    .unwrap_or_else(|e| panic!("error while reading {:?}: {:?}", &motion_path, e)),
            )
            .unwrap_or_else(|e| panic!("error while parsing {:?}: {:?}", &motion_path, e));
        }
    }
}
