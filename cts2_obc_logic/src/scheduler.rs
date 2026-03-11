#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    High = 4,
    Medium = 3,
    Low = 2,
    Debug = 1,
    None = 0,
}

const TOTAL_TASKS: usize = 256;

#[derive(Debug, Copy, Clone)]
pub enum TaskArgs {
    None,
    Message(&'static str),
    TwoU32(u32, u32),
}

pub type TaskFn = fn(TaskArgs);

#[derive(Debug, Copy, Clone)]
pub struct Task {
    pub name: &'static str,
    pub execute: TaskFn,
    pub args: TaskArgs,
    pub priority: Priority,
}
impl Task {
    pub fn new() -> Self {
        Task {
            name: "",
            execute: none,
            args: TaskArgs::None,
            priority: Priority::None,
        }
    }
}

pub struct Scheduler {
    high_level_tasks: [Task; TOTAL_TASKS],
    medium_level_tasks: [Task; TOTAL_TASKS],
    low_level_tasks: [Task; TOTAL_TASKS],
    debug_level_tasks: [Task; TOTAL_TASKS],
    head_high: usize,
    head_medium: usize,
    head_low: usize,
    head_debug: usize,
    tail_high: usize,
    tail_medium: usize,
    tail_low: usize,
    tail_debug: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            high_level_tasks: [Task::new(); TOTAL_TASKS],
            medium_level_tasks: [Task::new(); TOTAL_TASKS],
            low_level_tasks: [Task::new(); TOTAL_TASKS],
            debug_level_tasks: [Task::new(); TOTAL_TASKS],
            head_high: 0,
            head_medium: 0,
            head_low: 0,
            head_debug: 0,
            tail_high: 0,
            tail_medium: 0,
            tail_low: 0,
            tail_debug: 0,
        }
    }

    pub fn add_task(&mut self, mut task: Task, priority: Priority) -> Result<(), ()> {
        task.priority = priority;

        match priority {
            Priority::High => {
                if (self.tail_high + 1) % TOTAL_TASKS == self.head_high {
                    return Err(()); // Queue is full
                }
                self.high_level_tasks[self.tail_high] = task;
                self.tail_high = (self.tail_high + 1) % TOTAL_TASKS;
            }
            Priority::Medium => {
                if (self.tail_medium + 1) % TOTAL_TASKS == self.head_medium {
                    return Err(()); // Queue is full
                }
                self.medium_level_tasks[self.tail_medium] = task;
                self.tail_medium = (self.tail_medium + 1) % TOTAL_TASKS;
            }
            Priority::Low => {
                if (self.tail_low + 1) % TOTAL_TASKS == self.head_low {
                    return Err(()); // Queue is full
                }
                self.low_level_tasks[self.tail_low] = task;
                self.tail_low = (self.tail_low + 1) % TOTAL_TASKS;
            }
            Priority::Debug => {
                if (self.tail_debug + 1) % TOTAL_TASKS == self.head_debug {
                    return Err(()); // Queue is full
                }
                self.debug_level_tasks[self.tail_debug] = task;
                self.tail_debug = (self.tail_debug + 1) % TOTAL_TASKS;
            }
            Priority::None => return Err(()),
        }
        Ok(())
    }

    pub fn run_next_task(&mut self) -> Result<Task, ()> {
        let task;
        if !self.is_empty(Priority::High) {
            task = self.high_level_tasks[self.head_high];
            (task.execute)(task.args);
            self.remove_head_task(Priority::High)?;
        } else if !self.is_empty(Priority::Medium) {
            task = self.medium_level_tasks[self.head_medium];
            (task.execute)(task.args);
            self.remove_head_task(Priority::Medium)?;
        } else if !self.is_empty(Priority::Low) {
            task = self.low_level_tasks[self.head_low];
            (task.execute)(task.args);
            self.remove_head_task(Priority::Low)?;
        } else if !self.is_empty(Priority::Debug) {
            task = self.debug_level_tasks[self.head_debug];
            (task.execute)(task.args);
            self.remove_head_task(Priority::Debug)?;
        } else {
            return Err(());
        }

        Ok(task)
    }

    fn remove_head_task(&mut self, priority: Priority) -> Result<(), ()> {
        match priority {
            Priority::High => {
                self.high_level_tasks[self.head_high] = Task::new();
                self.head_high = (self.head_high + 1) % TOTAL_TASKS;
            }
            Priority::Medium => {
                self.medium_level_tasks[self.head_medium] = Task::new();
                self.head_medium = (self.head_medium + 1) % TOTAL_TASKS;
            }
            Priority::Low => {
                self.low_level_tasks[self.head_low] = Task::new();
                self.head_low = (self.head_low + 1) % TOTAL_TASKS;
            }
            Priority::Debug => {
                self.debug_level_tasks[self.head_debug] = Task::new();
                self.head_debug = (self.head_debug + 1) % TOTAL_TASKS;
            }
            Priority::None => return Err(()),
        }
        Ok(())
    }

    pub fn is_empty(&self, priority: Priority) -> bool {
        match priority {
            Priority::High => {
                self.head_high == self.tail_high
                    && self.high_level_tasks[self.head_high].priority == Priority::None
            }
            Priority::Medium => {
                self.head_medium == self.tail_medium
                    && self.medium_level_tasks[self.head_medium].priority == Priority::None
            }
            Priority::Low => {
                self.head_low == self.tail_low
                    && self.low_level_tasks[self.head_low].priority == Priority::None
            }
            Priority::Debug => {
                self.head_debug == self.tail_debug
                    && self.debug_level_tasks[self.head_debug].priority == Priority::None
            }
            Priority::None => true,
        }
    }
}

fn none(_: TaskArgs) {}
