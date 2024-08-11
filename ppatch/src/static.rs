use field_metadata::{load_fb_repo, ArchivedFieldBlockRepo};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref FIELD_BLOCK_REPO: &'static ArchivedFieldBlockRepo =
        unsafe { load_fb_repo(include_bytes!("../field_blocks.bin")) };
}
