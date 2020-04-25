use super::quaternion::Quaternion;
use super::r3::R3;

#[derive(Copy, Clone, Debug)]
pub struct Pose {
    pub pos: R3,
    pub orientation: Quaternion,
}

impl Pose {
    pub fn rotate(&self, origin: R3, rotation: Quaternion) -> Pose {
        Pose {
            pos: rotation.rotate(&(self.pos - origin)) + origin,
            orientation: rotation * self.orientation,
        }
    }
}