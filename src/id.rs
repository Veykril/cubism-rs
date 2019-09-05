#![allow(missing_docs)]
// https://docs.live2d.com/cubism-editor-manual/standard-parametor-list/#

/// Common Group IDs
pub mod groups {
    pub static EYE_BLINK: &str = "EyeBlink";
    pub static LIP_SYNC: &str = "LipSync";
}

/// Standard Part IDs
pub mod parts {
    pub static HIT_AREA_PREFIX: &str = "HitArea";
    pub static HIT_AREA_HEAD: &str = "Head";
    pub static HIT_AREA_BODY: &str = "Body";

    pub static CORE: &str = "Parts01Core";

    pub static ARM_PREFIX: &str = "Parts01Arm_";
    pub static ARM_L_PREFIX: &str = "Parts01ArmL_";
    pub static ARM_R_PREFIX: &str = "Parts01ArmR_";
}

/// Standard Parameter IDs
pub mod param {
    pub static ANGLE_X: &str = "ParamAngleX";
    pub static ANGLE_Y: &str = "ParamAngleY";
    pub static ANGLE_Z: &str = "ParamAngleZ";

    pub static EYE_L_OPEN: &str = "ParamEyeLOpen";
    pub static EYE_L_SMILE: &str = "ParamEyeLSmile";
    pub static EYE_R_OPEN: &str = "ParamEyeROpen";
    pub static EYE_R_SMILE: &str = "ParamEyeRSmile";
    pub static EYE_BALL_X: &str = "ParamEyeBallX";
    pub static EYE_BALL_Y: &str = "ParamEyeBallY";
    pub static EYE_BALL_FORM: &str = "ParamEyeBallForm";

    pub static BROW_LY: &str = "ParamBrowLY";
    pub static BROW_RY: &str = "ParamBrowRY";
    pub static BROW_LX: &str = "ParamBrowLX";
    pub static BROW_RX: &str = "ParamBrowRX";
    pub static BROW_L_ANGLE: &str = "ParamBrowLAngle";
    pub static BROW_R_ANGLE: &str = "ParamBrowRAngle";
    pub static BROW_L_FORM: &str = "ParamBrowLForm";
    pub static BROW_R_FORM: &str = "ParamBrowRForm";

    pub static MOUTH_FORM: &str = "ParamMouthForm";
    pub static MOUTH_OPEN_Y: &str = "ParamMouthOpenY";

    pub static CHEEK: &str = "ParamCheek";

    pub static BODY_ANGLE_X: &str = "ParamBodyAngleX";
    pub static BODY_ANGLE_Y: &str = "ParamBodyAngleY";
    pub static BODY_ANGLE_Z: &str = "ParamBodyAngleZ";

    pub static BREATH: &str = "ParamBreath";

    pub static ARM_LA: &str = "ParamArmLA";
    pub static ARM_RA: &str = "ParamArmRA";
    pub static ARM_LB: &str = "ParamArmLB";
    pub static ARM_RB: &str = "ParamArmRB";

    pub static HAND_L: &str = "ParamHandL";
    pub static HAND_R: &str = "ParamHandR";

    pub static HAIR_FRONT: &str = "ParamHairFront";
    pub static HAIR_SIDE: &str = "ParamHairSide";
    pub static HAIR_BACK: &str = "ParamHairBack";
    pub static HAIR_FLUFFY: &str = "ParamHairFluffy";

    pub static SHOULDER_Y: &str = "ParamShoulderY";

    pub static BUST_X: &str = "ParamBustX";
    pub static BUST_Y: &str = "ParamBustY";
    pub static BASE_X: &str = "ParamBaseX";
    pub static BASE_Y: &str = "ParamBaseY";

    pub static NONE: &str = "NONE:";
}
