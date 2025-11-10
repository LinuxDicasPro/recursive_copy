#[derive(Clone, Debug)]
pub struct CopyOptions {
    pub overwrite: bool,
    pub restrict_symlinks: bool,
    pub follow_symlinks: bool,
    pub content_only: bool,
    pub buffer_size: usize,
    pub depth: usize,
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            overwrite: false,
            restrict_symlinks: false,
            follow_symlinks: false,
            content_only: false,
            buffer_size: 64 * 1024,
            depth: 512,
        }
    }
}
