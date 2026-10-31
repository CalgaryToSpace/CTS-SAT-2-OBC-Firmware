use crate::error::SchedulerError;
use cts2_obc_telecommands::{EXECUTE_IMMEDIATELY, UnixTimestampMs, error::ExecuteCommandErr};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    High = 4,
    Medium = 3,
    Low = 2,
    Debug = 1,
    None = 0,
}

const TOTAL_TASKS: usize = 16;
const DELAYED_TASKS: usize = 16;

pub type TaskFn = fn(&str) -> Result<(), ExecuteCommandErr>;

#[derive(Debug, Clone)]
pub struct Task {
    pub name: &'static str,
    pub execute: TaskFn,
    pub args: heapless::String<256>,
    pub priority: Priority,
    pub tsexec: UnixTimestampMs,
}
impl Task {
    pub const fn new() -> Self {
        Task {
            name: "",
            execute: none,
            args: heapless::String::new(),
            priority: Priority::None,
            tsexec: EXECUTE_IMMEDIATELY,
        }
    }
}

impl Default for Task {
    fn default() -> Self {
        Self::new()
    }
}

struct TaskQueue {
    tasks: [Task; TOTAL_TASKS],
    head: usize,
    tail: usize,
    count: usize,
}

impl TaskQueue {
    const fn new() -> Self {
        const DEFAULT_TASK: Task = Task::new();
        Self {
            tasks: [DEFAULT_TASK; TOTAL_TASKS],
            head: 0,
            tail: 0,
            count: 0,
        }
    }
}

pub struct Scheduler {
    delayed_tasks: [Task; DELAYED_TASKS],
    high_level_tasks: TaskQueue,
    medium_level_tasks: TaskQueue,
    low_level_tasks: TaskQueue,
    debug_level_tasks: TaskQueue,
}

impl Scheduler {
    pub const fn new() -> Self {
        const DEFAULT_TASK: Task = Task::new();
        Scheduler {
            delayed_tasks: [DEFAULT_TASK; DELAYED_TASKS],
            high_level_tasks: TaskQueue::new(),
            medium_level_tasks: TaskQueue::new(),
            low_level_tasks: TaskQueue::new(),
            debug_level_tasks: TaskQueue::new(),
        }
    }

    pub fn release_task(&mut self, unix_ms_ts: UnixTimestampMs) {
        for i in 0..DELAYED_TASKS {
            let task = &self.delayed_tasks[i];
            if task.priority != Priority::None
                && task.tsexec <= unix_ms_ts
                && self.add_ready_task(task.clone()).is_ok()
            {
                self.delayed_tasks[i] = Task::new();
            }
        }
    }

    fn queue_mut(&mut self, priority: Priority) -> Result<&mut TaskQueue, SchedulerError> {
        match priority {
            Priority::High => Ok(&mut self.high_level_tasks),
            Priority::Medium => Ok(&mut self.medium_level_tasks),
            Priority::Low => Ok(&mut self.low_level_tasks),
            Priority::Debug => Ok(&mut self.debug_level_tasks),
            Priority::None => Err(SchedulerError::InvalidPriority),
        }
    }

    pub fn add_ready_task(&mut self, task: Task) -> Result<(), SchedulerError> {
        let queue = self.queue_mut(task.priority)?;
        if queue.count == TOTAL_TASKS {
            return Err(SchedulerError::QueueFull);
        }
        queue.tasks[queue.tail] = task;
        queue.tail = (queue.tail + 1) % TOTAL_TASKS;
        queue.count += 1;
        Ok(())
    }

    pub fn add_task(&mut self, task: Task) -> Result<(), SchedulerError> {
        if task.priority == Priority::None {
            return Err(SchedulerError::InvalidPriority);
        }

        // Scheduled tasks
        if task.tsexec != EXECUTE_IMMEDIATELY {
            let slot = self
                .delayed_tasks
                .iter_mut()
                .find(|task| task.priority == Priority::None)
                .ok_or(SchedulerError::QueueFull)?;
            *slot = task;
            return Ok(());
        }

        // Immediate tasks
        self.add_ready_task(task)
    }

    pub fn run_next_task(
        &mut self,
    ) -> Result<(Task, Result<(), ExecuteCommandErr>), SchedulerError> {
        const PRIORITIES: [Priority; 4] = [
            Priority::High,
            Priority::Medium,
            Priority::Low,
            Priority::Debug,
        ];

        for priority in PRIORITIES {
            let queue = self.queue_mut(priority)?;
            if queue.count == 0 {
                continue;
            }
            let task = queue.tasks[queue.head].clone();
            let result = (task.execute)(task.args.as_str());
            self.remove_head_task(priority)?;
            return Ok((task, result));
        }
        Err(SchedulerError::NoTasksAvailable)
    }

    fn remove_head_task(&mut self, priority: Priority) -> Result<(), SchedulerError> {
        let queue = self.queue_mut(priority)?;
        queue.tasks[queue.head] = Task::new();
        queue.head = (queue.head + 1) % TOTAL_TASKS;
        queue.count -= 1;
        Ok(())
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

fn none(_: &str) -> Result<(), ExecuteCommandErr> {
    Ok(())
}
