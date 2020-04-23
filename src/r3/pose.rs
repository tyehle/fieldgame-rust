use super::quaternion::Quaternion;
use super::r3::R3;

#[derive(Copy, Clone, Debug)]
pub struct Pose {
    pub pos: R3,
    pub orientation: Quaternion,
}
