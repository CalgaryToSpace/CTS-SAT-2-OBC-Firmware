#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;
pub mod error;
pub mod scheduler;

// TODO: Remove this placeholder function and add testable logic parts in here.
pub fn multiply_by_2(i: u32) -> u32 {
    i * 2
}

#[cfg(test)]
mod tests {
    use cts2_obc_telecommands::{EXECUTE_IMMEDIATELY, error::ExecuteCommandErr};

    use super::*;

    fn test_print_task(args: &str) -> Result<(), ExecuteCommandErr> {
        assert!(!args.is_empty());
        Ok(())
    }

    fn test_args_task(args: &str) -> Result<(), ExecuteCommandErr> {
        assert_eq!(args, "20,5");
        Ok(())
    }

    #[test]
    fn test_placeholder() {
        assert_eq!(multiply_by_2(21), 42);
        assert_eq!(multiply_by_2(0), 0);
    }

    #[test]
    fn test_scheduler_creation() {
        let mut sched = scheduler::Scheduler::new();
        assert!(matches!(
            sched.run_next_task(),
            Err(error::SchedulerError::NoTasksAvailable)
        ));
    }

    #[test]
    fn test_scheduler_add_and_run_task() {
        let mut sched = scheduler::Scheduler::new();
        let task = scheduler::Task {
            name: "Test Task",
            execute: test_print_task,
            args: heapless::String::try_from("Executing test task").unwrap(),
            priority: scheduler::Priority::High,
            tsexec: EXECUTE_IMMEDIATELY,
        };
        assert!(sched.add_task(task).is_ok());
        assert!(sched.run_next_task().unwrap().1.is_ok());
    }

    #[test]
    fn test_scheduler_empty_run() {
        let mut sched = scheduler::Scheduler::new();
        assert!(sched.run_next_task().is_err());
    }

    #[test]
    fn priority_scheduling() {
        let mut sched = scheduler::Scheduler::new();
        let low_task = scheduler::Task {
            name: "Low Priority Task",
            execute: test_print_task,
            args: heapless::String::try_from("Executing low priority task").unwrap(),
            priority: scheduler::Priority::Low,
            tsexec: EXECUTE_IMMEDIATELY,
        };
        let high_task = scheduler::Task {
            name: "High Priority Task",
            execute: test_print_task,
            args: heapless::String::try_from("Executing high priority task").unwrap(),
            priority: scheduler::Priority::High,
            tsexec: EXECUTE_IMMEDIATELY,
        };
        assert!(sched.add_task(low_task).is_ok());
        assert!(sched.add_task(high_task).is_ok());
        // High priority task should run first
        assert!(sched.run_next_task().is_ok_and(
            |(task, result)| result.is_ok() && task.priority == scheduler::Priority::High
        ));
        // Then the low priority task
        assert!(sched.run_next_task().is_ok_and(
            |(task, result)| result.is_ok() && task.priority == scheduler::Priority::Low
        ));
    }

    #[test]
    fn test_function_with_arguments() {
        let mut sched = scheduler::Scheduler::new();
        let task = scheduler::Task {
            name: "Task with Args",
            execute: test_args_task,
            args: heapless::String::try_from("20,5").unwrap(),
            priority: scheduler::Priority::Medium,
            tsexec: EXECUTE_IMMEDIATELY,
        };
        assert!(sched.add_task(task).is_ok());
        assert!(sched.run_next_task().unwrap().1.is_ok());
    }

    #[test]
    fn test_queue_full() {
        let mut sched = scheduler::Scheduler::new();
        for _i in 0..16 {
            let task = scheduler::Task {
                name: "Filler Task",
                execute: test_print_task,
                args: heapless::String::try_from("Filling queue").unwrap(),
                priority: scheduler::Priority::Debug,
                tsexec: EXECUTE_IMMEDIATELY,
            };
            assert!(sched.add_task(task).is_ok());
        }
        // Now the queue should be full
        let extra_task = scheduler::Task {
            name: "Extra Task",
            execute: test_print_task,
            args: heapless::String::try_from("This should fail").unwrap(),
            priority: scheduler::Priority::Debug,
            tsexec: EXECUTE_IMMEDIATELY,
        };
        assert!(sched.add_task(extra_task).is_err());
    }

    #[test]
    fn test_scheduled_task() {
        let mut sched = scheduler::Scheduler::new();
        let scheduled_task = scheduler::Task {
            name: "Scheduled Task",
            execute: test_print_task,
            args: heapless::String::try_from("This is a scheduled task").unwrap(),
            priority: scheduler::Priority::Medium,
            tsexec: 123,
        };
        assert!(sched.add_task(scheduled_task).is_ok());

        // The scheduled task should not run immediately
        assert!(sched.run_next_task().is_err());

        sched.release_task(124);

        // Now the scheduled task should run
        assert!(sched.run_next_task().unwrap().1.is_ok());
    }
}
