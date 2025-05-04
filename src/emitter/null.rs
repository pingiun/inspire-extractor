use crate::FeatureMember;

use super::FeatureMemberEmitter;

pub struct NullEmitter;

impl NullEmitter {
    pub fn new() -> Self {
        NullEmitter
    }
}

impl Default for NullEmitter {
    fn default() -> Self {
        Self::new()
    }
}
impl FeatureMemberEmitter for NullEmitter {
    fn emit(&mut self, _feature_member: FeatureMember) {
        // No operation
    }

    fn flush(&mut self) {
        // No operation
    }
}
