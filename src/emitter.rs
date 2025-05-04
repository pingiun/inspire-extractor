pub mod multifile;
pub mod null;

use crate::FeatureMember;

pub trait FeatureMemberEmitter {
    fn emit(&mut self, feature_member: FeatureMember);
    fn flush(&mut self);
}

pub enum ChooseEmitter {
    MultiFile(multifile::MultiFileEmitter),
    Null(null::NullEmitter),
}

impl From<multifile::MultiFileEmitter> for ChooseEmitter {
    fn from(emitter: multifile::MultiFileEmitter) -> Self {
        ChooseEmitter::MultiFile(emitter)
    }
}
impl From<null::NullEmitter> for ChooseEmitter {
    fn from(emitter: null::NullEmitter) -> Self {
        ChooseEmitter::Null(emitter)
    }
}
impl FeatureMemberEmitter for ChooseEmitter {
    fn emit(&mut self, feature_member: FeatureMember) {
        match self {
            ChooseEmitter::MultiFile(emitter) => emitter.emit(feature_member),
            ChooseEmitter::Null(emitter) => emitter.emit(feature_member),
        }
    }

    fn flush(&mut self) {
        match self {
            ChooseEmitter::MultiFile(emitter) => emitter.flush(),
            ChooseEmitter::Null(emitter) => emitter.flush(),
        }
    }
}

impl Default for ChooseEmitter {
    fn default() -> Self {
        ChooseEmitter::Null(null::NullEmitter::default())
    }
}
