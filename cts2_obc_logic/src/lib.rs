#![cfg_attr(not(test), no_std)]

#[cfg(test)]
extern crate std;
pub mod scheduler;

// TODO: Remove this placeholder function and add testable logic parts in here.
pub fn multiply_by_2(i: u32) -> u32 {
    i * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_print_task(args: scheduler::TaskArgs) {
        match args {
            scheduler::TaskArgs::Message(message) => assert!(!message.is_empty()),
            _ => panic!("expected TaskArgs::Message"),
        }
    }

    fn test_sum_task(args: scheduler::TaskArgs) {
        match args {
            scheduler::TaskArgs::TwoU32(a, b) => {
                let _ = a + b;
            }
            _ => panic!("expected TaskArgs::TwoU32"),
        }
    }

    #[test]
    fn test_placeholder() {
        assert_eq!(multiply_by_2(21), 42);
        assert_eq!(multiply_by_2(0), 0);
    }

    #[test]
    fn test_scheduler_creation() {
        let sched = scheduler::Scheduler::new();
        assert!(sched.is_empty(scheduler::Priority::High));
        assert!(sched.is_empty(scheduler::Priority::Medium));
        assert!(sched.is_empty(scheduler::Priority::Low));
        assert!(sched.is_empty(scheduler::Priority::Debug));
    }

    #[test]
    fn test_scheduler_add_and_run_task() {
        let mut sched = scheduler::Scheduler::new();
        let task = scheduler::Task {
            name: "Test Task",
            execute: test_print_task,
            args: scheduler::TaskArgs::Message("Executing test task"),
            priority: scheduler::Priority::High,
        };
        assert!(sched.add_task(task, scheduler::Priority::High).is_ok());
        assert!(sched.run_next_task().is_ok());
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
            args: scheduler::TaskArgs::Message("Executing low priority task"),
            priority: scheduler::Priority::Low,
        };
        let high_task = scheduler::Task {
            name: "High Priority Task",
            execute: test_print_task,
            args: scheduler::TaskArgs::Message("Executing high priority task"),
            priority: scheduler::Priority::High,
        };
        assert!(sched.add_task(low_task, scheduler::Priority::Low).is_ok());
        assert!(sched.add_task(high_task, scheduler::Priority::High).is_ok());
        // High priority task should run first
        assert!(
            sched
                .run_next_task()
                .is_ok_and(|x| x.priority == scheduler::Priority::High)
        );
        // Then the low priority task
        assert!(
            sched
                .run_next_task()
                .is_ok_and(|x| x.priority == scheduler::Priority::Low)
        );
    }

    #[test]
    fn test_function_with_arguments() {
        let mut sched = scheduler::Scheduler::new();
        let task = scheduler::Task {
            name: "Task with Args",
            execute: test_sum_task,
            args: scheduler::TaskArgs::TwoU32(42, 7),
            priority: scheduler::Priority::Medium,
        };
        assert!(sched.add_task(task, scheduler::Priority::Medium).is_ok());
        assert!(sched.run_next_task().is_ok());
    }

    #[test]
    fn test_queue_full() {
        let mut sched = scheduler::Scheduler::new();
        for _i in 0..256 {
            let task = scheduler::Task {
                name: "Filler Task",
                execute: test_print_task,
                args: scheduler::TaskArgs::Message("Filling queue"),
                priority: scheduler::Priority::Debug,
            };
            assert!(sched.add_task(task, scheduler::Priority::Debug).is_ok());
        }
        // Now the queue should be full
        let extra_task = scheduler::Task {
            name: "Extra Task",
            execute: test_print_task,
            args: scheduler::TaskArgs::Message("This should fail"),
            priority: scheduler::Priority::Debug,
        };
        assert!(
            sched
                .add_task(extra_task, scheduler::Priority::Debug)
                .is_err()
        );
    }
}
