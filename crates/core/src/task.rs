use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};
use uuid::Uuid;

use crate::{
  MediaData,
  ProcessingPipeline,
  ProcessingResult,
  ellastic_errors::{
    EllasticError,
    Result,
  },
};

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
  Pending,
  Running,
  Completed,
  Failed,
  Cancelled,
}

#[derive(Debug, Clone)]
pub enum TaskPriority {
  Low,
  Normal,
  High,
  Critical,
}

impl TaskPriority {
  pub fn value(&self) -> u8 {
    match self {
      TaskPriority::Low => 1,
      TaskPriority::Normal => 2,
      TaskPriority::High => 3,
      TaskPriority::Critical => 4,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Task {
  pub id: Uuid,
  pub name: String,
  pub pipeline_id: Uuid,
  pub media_id: Uuid,
  pub priority: TaskPriority,
  pub status: TaskStatus,
  pub created_at: std::time::SystemTime,
  pub started_at: Option<std::time::SystemTime>,
  pub completed_at: Option<std::time::SystemTime>,
  pub result: Option<ProcessingResult>,
  pub error: Option<String>,
}

impl Task {
  pub fn new(name: String, pipeline_id: Uuid, media_id: Uuid, priority: TaskPriority) -> Self {
    Self {
      id: Uuid::new_v4(),
      name,
      pipeline_id,
      media_id,
      priority,
      status: TaskStatus::Pending,
      created_at: std::time::SystemTime::now(),
      started_at: None,
      completed_at: None,
      result: None,
      error: None,
    }
  }

  pub fn start(&mut self) {
    self.status = TaskStatus::Running;
    self.started_at = Some(std::time::SystemTime::now());
  }

  pub fn complete(&mut self, result: ProcessingResult) {
    self.status = TaskStatus::Completed;
    self.completed_at = Some(std::time::SystemTime::now());
    self.result = Some(result);
  }

  pub fn fail(&mut self, error: String) {
    self.status = TaskStatus::Failed;
    self.completed_at = Some(std::time::SystemTime::now());
    self.error = Some(error);
  }

  pub fn cancel(&mut self) {
    self.status = TaskStatus::Cancelled;
    self.completed_at = Some(std::time::SystemTime::now());
  }

  pub fn duration(&self) -> Option<std::time::Duration> {
    match (self.started_at, self.completed_at) {
      (Some(start), Some(end)) => end.duration_since(start).ok(),
      _ => None,
    }
  }
}

#[derive(Debug)]
pub struct TaskManager {
  tasks: Arc<Mutex<HashMap<Uuid, Task>>>,
  queue: Arc<Mutex<Vec<Uuid>>>,
}

impl TaskManager {
  pub fn new() -> Self {
    Self {
      tasks: Arc::new(Mutex::new(HashMap::new())),
      queue: Arc::new(Mutex::new(Vec::new())),
    }
  }

  pub fn create_task(
    &self,
    name: String,
    pipeline_id: Uuid,
    media_id: Uuid,
    priority: TaskPriority,
  ) -> Uuid {
    let task = Task::new(name, pipeline_id, media_id, priority);
    let task_id = task.id;

    {
      let mut tasks = self.tasks.lock().unwrap();
      tasks.insert(task_id, task);
    }

    {
      let mut queue = self.queue.lock().unwrap();
      queue.push(task_id);
      queue.sort_by(|&a, &b| {
        let tasks = self.tasks.lock().unwrap();
        let task_a = tasks.get(&a).unwrap();
        let task_b = tasks.get(&b).unwrap();

        task_a
          .priority
          .value()
          .cmp(&task_b.priority.value())
          .then_with(|| task_a.created_at.cmp(&task_b.created_at))
      });
    }

    task_id
  }

  pub fn get_next_task(&self) -> Option<Task> {
    let mut queue = self.queue.lock().unwrap();

    while let Some(task_id) = queue.first() {
      let tasks = self.tasks.lock().unwrap();
      if let Some(task) = tasks.get(task_id) {
        if task.status == TaskStatus::Pending {
          let task_id = *task_id;
          drop(tasks);
          drop(queue);

          let mut tasks = self.tasks.lock().unwrap();
          if let Some(mut task) = tasks.remove(&task_id) {
            task.start();
            let task_clone = task.clone();
            tasks.insert(task_id, task);
            return Some(task_clone);
          }
        }
      }
      queue.remove(0);
    }

    None
  }

  pub fn complete_task(&self, task_id: Uuid, result: ProcessingResult) -> Result<()> {
    let mut tasks = self.tasks.lock().unwrap();
    if let Some(task) = tasks.get_mut(&task_id) {
      task.complete(result);
      Ok(())
    } else {
      Err(EllasticError::TaskNotFound(task_id))
    }
  }

  pub fn fail_task(&self, task_id: Uuid, error: String) -> Result<()> {
    let mut tasks = self.tasks.lock().unwrap();
    if let Some(task) = tasks.get_mut(&task_id) {
      task.fail(error);
      Ok(())
    } else {
      Err(EllasticError::TaskNotFound(task_id))
    }
  }

  pub fn cancel_task(&self, task_id: Uuid) -> Result<()> {
    let mut tasks = self.tasks.lock().unwrap();
    if let Some(task) = tasks.get_mut(&task_id) {
      task.cancel();

      let mut queue = self.queue.lock().unwrap();
      queue.retain(|&id| id != task_id);

      Ok(())
    } else {
      Err(EllasticError::TaskNotFound(task_id))
    }
  }

  pub fn get_task(&self, task_id: Uuid) -> Option<Task> {
    let tasks = self.tasks.lock().unwrap();
    tasks.get(&task_id).cloned()
  }

  pub fn list_tasks(&self) -> Vec<Task> {
    let tasks = self.tasks.lock().unwrap();
    tasks.values().cloned().collect()
  }

  pub fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<Task> {
    let tasks = self.tasks.lock().unwrap();
    tasks
      .values()
      .filter(|task| task.status == status)
      .cloned()
      .collect()
  }

  pub fn clear_completed(&self) -> usize {
    let mut tasks = self.tasks.lock().unwrap();
    let mut queue = self.queue.lock().unwrap();

    let completed_ids: Vec<Uuid> = tasks
      .iter()
      .filter(|(_, task)| {
        matches!(
          task.status,
          TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
      })
      .map(|(id, _)| *id)
      .collect();

    let count = completed_ids.len();

    for id in &completed_ids {
      tasks.remove(id);
      queue.retain(|&task_id| task_id != *id);
    }

    count
  }

  pub fn task_count(&self) -> usize {
    let tasks = self.tasks.lock().unwrap();
    tasks.len()
  }

  pub fn pending_count(&self) -> usize {
    let tasks = self.tasks.lock().unwrap();
    tasks
      .values()
      .filter(|task| task.status == TaskStatus::Pending)
      .count()
  }

  pub fn running_count(&self) -> usize {
    let tasks = self.tasks.lock().unwrap();
    tasks
      .values()
      .filter(|task| task.status == TaskStatus::Running)
      .count()
  }
}

impl Default for TaskManager {
  fn default() -> Self {
    Self::new()
  }
}
