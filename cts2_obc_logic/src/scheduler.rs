#[derive(Debug, Copy, Clone)]
pub enum Priority {
    High = 4,
    Medium = 3,
    Low = 2,
    Debug = 1,
    None = 0,
}
#[derive(Debug, Copy, Clone)]
pub struct Task {
    pub name: &'static str,
    pub execute: fn(),
    pub num_args: usize,
    pub args: (),
    pub priority: Priority,
}
impl Task {
    pub fn new() -> Self {
        Task {
            name: "",
            execute: none,
            num_args: 0,
            args: (),
            priority: Priority::None,
        }
    }
}

struct Scheduler {
    high_level_tasks: [Option<Task>; 256],
    medium_level_tasks: [Option<Task>; 256],
    low_level_tasks: [Option<Task>; 256],
    debug_level_tasks: [Option<Task>; 256],
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
            high_level_tasks: [Some(Task::new()); 256],
            medium_level_tasks: [Some(Task::new()); 256],
            low_level_tasks: [Some(Task::new()); 256],
            debug_level_tasks: [Some(Task::new()); 256],
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

    pub fn add_task(&mut self, task: Task, priority: Priority) -> Result<(), ()> {
        match priority {
            Priority::High => {
                self.high_level_tasks[self.tail_high].unwrap().name = task.name;
                self.tail_high = (self.tail_high + 1) % 256;
            }
            Priority::Medium => {
                self.medium_level_tasks[self.tail_medium].unwrap().name = task.name;
                self.tail_medium = (self.tail_medium + 1) % 256;
            }
            Priority::Low => {
                self.low_level_tasks[self.tail_low].unwrap().name = task.name;
                self.tail_low = (self.tail_low + 1) % 256;
            }
            Priority::Debug => {
                self.debug_level_tasks[self.tail_debug].unwrap().name = task.name;
                self.tail_debug = (self.tail_debug + 1) % 256;
            }
            Priority::None => return Err(()),
        }
        Ok(())
    }

    pub fn run_next_task(&mut self) -> Result<(), ()> {
        if !self.is_empty(Priority::High) {
            (self.high_level_tasks[self.head_high].unwrap().execute)();
            self.remove_head_task(Priority::High)?;
        } else if !self.is_empty(Priority::Medium) {
            (self.medium_level_tasks[self.head_medium].unwrap().execute)();
            self.remove_head_task(Priority::Medium)?;
        } else if !self.is_empty(Priority::Low) {
            (self.low_level_tasks[self.head_low].unwrap().execute)();
            self.remove_head_task(Priority::Low)?;
        } else if !self.is_empty(Priority::Debug) {
            (self.debug_level_tasks[self.head_debug].unwrap().execute)();
            self.remove_head_task(Priority::Debug)?;
        } else {
            return Err(());
        }

        Ok(())
    }

    fn remove_head_task(&mut self, priority: Priority) -> Result<(), ()> {
        match priority {
            Priority::High => {
                self.high_level_tasks[self.head_high].unwrap().name = "";
                self.high_level_tasks[self.head_high].unwrap().execute = none;
                self.high_level_tasks[self.head_high].unwrap().num_args = 0;
                self.high_level_tasks[self.head_high].unwrap().args = ();
                self.high_level_tasks[self.head_high].unwrap().priority = Priority::None;
                self.head_high = (self.head_high + 1) % 256;
            }
            Priority::Medium => {
                self.medium_level_tasks[self.head_medium].unwrap().name = "";
                self.medium_level_tasks[self.head_medium].unwrap().execute = none;
                self.medium_level_tasks[self.head_medium].unwrap().num_args = 0;
                self.medium_level_tasks[self.head_medium].unwrap().args = ();
                self.medium_level_tasks[self.head_medium].unwrap().priority = Priority::None;
                self.head_medium = (self.head_medium + 1) % 256;
            }
            Priority::Low => {
                self.low_level_tasks[self.head_low].unwrap().name = "";
                self.low_level_tasks[self.head_low].unwrap().execute = none;
                self.low_level_tasks[self.head_low].unwrap().num_args = 0;
                self.low_level_tasks[self.head_low].unwrap().args = ();
                self.low_level_tasks[self.head_low].unwrap().priority = Priority::None;
                self.head_low = (self.head_low + 1) % 256;
            }
            Priority::Debug => {
                self.debug_level_tasks[self.head_debug].unwrap().name = "";
                self.debug_level_tasks[self.head_debug].unwrap().execute = none;
                self.debug_level_tasks[self.head_debug].unwrap().num_args = 0;
                self.debug_level_tasks[self.head_debug].unwrap().args = ();
                self.debug_level_tasks[self.head_debug].unwrap().priority = Priority::None;
                self.head_debug = (self.head_debug + 1) % 256;
            }
            Priority::None => return Err(()),
        }
        Ok(())
    }

    pub fn is_empty(&self, priority: Priority) -> bool {
        match priority {
            Priority::High => self.head_high == self.tail_high,
            Priority::Medium => self.head_medium == self.tail_medium,
            Priority::Low => self.head_low == self.tail_low,
            Priority::Debug => self.head_debug == self.tail_debug,
            Priority::None => true,
        }
    }
}
fn none() {}
