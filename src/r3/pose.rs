
use super::r3::R3;
use super::quaternion::Quaternion;

#[derive(Copy, Clone, Debug)]
pub struct Pose {
    pub pos: R3,
    pub orientation: Quaternion,
}