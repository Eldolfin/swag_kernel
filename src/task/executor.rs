use core::task::{Waker, Context, Poll};

use alloc::{collections::BTreeMap, sync::Arc, format, task::Wake, string::String};
use crossbeam_queue::ArrayQueue;
use lazy_static::lazy_static;

use super::{TaskId, Task};

/// aka MAX_TASKS
const TASK_QUEUE_CAPACITY: usize = 100;

lazy_static!{
    static ref TASK_QUEUE_FULL_MESSAGE: String = format!(
    "Task Queue full. Increase TASK_QUEUE_CAPACITY (currently {TASK_QUEUE_CAPACITY})");
}

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl Executor {
    pub fn new() -> Self {
        Self { 
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(TASK_QUEUE_CAPACITY)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID ({:?}) already in tasks", task_id);
        }
        self.task_queue.push(task_id).expect(&TASK_QUEUE_FULL_MESSAGE);
    }

    fn run_ready_tasks(&mut self) {
        while let Some(task_id) = self.task_queue.pop() {
            let task = match self.tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };
            let waker = self.waker_cache
                .entry(task_id)
                .or_insert_with(|| 
                    TaskWaker::new_waker(task_id, self.task_queue.clone())
                );
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done so remove it and its cached waker
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                },
                Poll::Pending => {},
            }
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts;

        interrupts::disable();
        if self.task_queue.is_empty() {
            interrupts::enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskWaker {
    fn new_waker(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(Self {
            task_id,
            task_queue,
        }))
    } 

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect(&TASK_QUEUE_FULL_MESSAGE);
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
