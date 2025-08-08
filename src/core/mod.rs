pub mod analyzer;
pub mod batch;
pub mod processor;

pub use batch::BatchProcessor;
pub use processor::analyze_and_process_mkv_file;