use thiserror::*;

#[derive(Debug, Error)]
pub enum CstErr {
    #[error("an empty source file was provided")]
    EmptySourceFile,
}
