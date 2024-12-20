use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete, patch},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    id: u64,
    description: String,
    completed: bool,
}

impl Task {
    fn new(id: u64, description: String) -> Self {
        Self {
            id,
            description,
            completed: false,
        }
    }
}

type SharedState = Arc<TokioMutex<Vec<Task>>>;

#[derive(Deserialize)]
struct AddTask {
    description: String,
}

// Добавить несколько задач
async fn add_tasks(
    State(state): State<SharedState>,
    Json(payload): Json<Vec<AddTask>>,
) -> Json<Vec<Task>> {
    let mut tasks = state.lock().await;
    let mut new_tasks = vec![];

    for task in payload {
        let used_ids: HashSet<u64> = tasks.iter().map(|task| task.id).collect();
        let new_id = (1..).find(|id| !used_ids.contains(id)).unwrap();

        let new_task = Task::new(new_id, task.description);
        tasks.push(new_task.clone());
        new_tasks.push(new_task);
    }

    Json(new_tasks)
}

// Получить список задач
async fn list_tasks(State(state): State<SharedState>) -> Json<Vec<Task>> {
    let tasks = state.lock().await;
    Json(tasks.clone())
}

// Пометить задачи как выполненные
async fn complete_tasks(
    State(state): State<SharedState>,
    Json(ids): Json<Vec<u64>>,
) -> &'static str {
    let mut tasks = state.lock().await;
    let mut updated = false;

    for task in tasks.iter_mut() {
        if ids.contains(&task.id) {
            task.completed = true;
            updated = true;
        }
    }

    if updated {
        "Tasks marked as completed"
    } else {
        "No tasks found"
    }
}

// Удалить несколько задач
async fn delete_tasks(
    State(state): State<SharedState>,
    Json(ids): Json<Vec<u64>>,
) -> &'static str {
    let mut tasks = state.lock().await;
    let len_before = tasks.len();

    tasks.retain(|task| !ids.contains(&task.id));

    if tasks.len() < len_before {
        "Tasks deleted"
    } else {
        "No tasks found"
    }
}

#[tokio::main]
async fn main() {
    let shared_state: SharedState = Arc::new(TokioMutex::new(Vec::new()));

    let app = Router::new()
        .route("/tasks", post(add_tasks).get(list_tasks))
        .route("/tasks/batch-delete", delete(delete_tasks))
        .route("/tasks/batch-complete", patch(complete_tasks))
        .with_state(shared_state);

    println!("Сервер запущен на http://127.0.0.1:3000");
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}



//curl команды для тестирования
// добавление задач
// curl -X POST -H "Content-Type: application/json" -d '[{"description": "Task 1"}, {"description": "Task 2"}]' http://127.0.0.1:3000/tasks
// получение задач
// curl -X GET http://127.0.0.1:3000/tasks
// удаление задач
// curl -X DELETE -H "Content-Type: application/json" -d '[1, 2]' http://127.0.0.1:3000/tasks/batch-delete
// пометка задач как выполненные
// curl -X PATCH -H "Content-Type: application/json" -d '[1, 2]' http://127.0.0.1:3000/tasks/batch-complete