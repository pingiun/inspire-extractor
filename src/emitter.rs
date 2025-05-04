pub mod multifile;
pub mod null;
pub mod sqlite;

use crate::FeatureMember;

pub trait FeatureMemberEmitter {
    fn emit(&mut self, feature_member: FeatureMember);
    fn start(&mut self) {}
    fn end(&mut self) {}
}

pub enum ChooseEmitter {
    MultiFile(multifile::MultiFileEmitter),
    Null(null::NullEmitter),
    Sqlite(sqlite::SqliteEmitter),
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
impl From<sqlite::SqliteEmitter> for ChooseEmitter {
    fn from(emitter: sqlite::SqliteEmitter) -> Self {
        ChooseEmitter::Sqlite(emitter)
    }
}
impl FeatureMemberEmitter for ChooseEmitter {
    fn emit(&mut self, feature_member: FeatureMember) {
        match self {
            ChooseEmitter::MultiFile(emitter) => emitter.emit(feature_member),
            ChooseEmitter::Null(emitter) => emitter.emit(feature_member),
            ChooseEmitter::Sqlite(emitter) => emitter.emit(feature_member),
        }
    }
    fn start(&mut self) {
        match self {
            ChooseEmitter::MultiFile(emitter) => emitter.start(),
            ChooseEmitter::Null(emitter) => emitter.start(),
            ChooseEmitter::Sqlite(emitter) => emitter.start(),
        }
    }
    fn end(&mut self) {
        match self {
            ChooseEmitter::MultiFile(emitter) => emitter.end(),
            ChooseEmitter::Null(emitter) => emitter.end(),
            ChooseEmitter::Sqlite(emitter) => emitter.end(),
        }
    }
}

impl Default for ChooseEmitter {
    fn default() -> Self {
        ChooseEmitter::Null(null::NullEmitter::default())
    }
}
