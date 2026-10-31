use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum SchedulerError {
    #[error("Task queue is full")]
    QueueFull,

    #[error("Priority level is not specified (None)")]
    InvalidPriority,

    #[error("No tasks available to execute")]
    NoTasksAvailable,
}
