//! Bounded integration matrix against local dev-kit endpoints.

mod devkit;
mod fixtures;
mod lab;
mod matrix;
mod report;
mod server;

pub use devkit::{devkit_manifest_independent_of_tck, pinned_devkit_manifest_path};
pub use fixtures::{standard_fixtures, unavailable_fixtures};
pub use lab::FixtureVariant;
pub use matrix::{
    DEVKIT_REVISION, MatrixCase, MatrixMode, MatrixOutcome, PROFILE_VERSION, evidence_dir,
    run_invalid_configuration, run_matrix,
};
pub use report::{
    contract_report, full_suite_includes_agent_message, interface_differences,
    is_explained_mismatch_matrix_row, profile_has_video_cases, unexplained_interface_differences,
};
pub use server::FixtureMode;
