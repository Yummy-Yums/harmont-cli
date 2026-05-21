//! Local-first build orchestration.
//!
//! The orchestrator owns the per-run state: the event bus that
//! announces `BuildEvent`s, the source-archive store served to
//! step-executor plugins, the cancellation atomic, and the chain
//! scheduler that dispatches each step to a plugin via the plan-1
//! plugin host.

pub mod archive;
pub mod cache;
pub mod cancel;
pub mod docker_client;
pub mod docker_host_fns;
pub mod events;
pub mod graph;
pub mod output_subscriber;
pub mod scheduler;
pub mod source;
pub mod state;

pub use scheduler::run;
pub use state::OrchestratorState;
