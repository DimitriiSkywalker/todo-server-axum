use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{post, patch},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc};
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

// Добавить задачу с пересчётом id
async fn add_task(
    State(state): State<SharedState>,
    Json(payload): Json<AddTask>,
) -> Json<Task> {
    let mut tasks = state.lock().await;

    // Найти минимально доступный id
    let used_ids: HashSet<u64> = tasks.iter().map(|task| task.id).collect();
    let new_id = (1..).find(|id| !used_ids.contains(id)).unwrap();

    let new_task = Task::new(new_id, payload.description);
    tasks.push(new_task.clone());
    Json(new_task)
}

// Получить список задач
async fn list_tasks(State(state): State<SharedState>) -> Json<Vec<Task>> {
    let tasks = state.lock().await;
    Json(tasks.clone())
}

// Пометить задачу как выполненную
async fn complete_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> StatusCode {
    let mut tasks = state.lock().await;
    if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
        task.completed = true;
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

// Удалить задачу
async fn delete_task(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> StatusCode {
    let mut tasks = state.lock().await;
    let len_before = tasks.len();
    tasks.retain(|task| task.id != id);
    if tasks.len() < len_before {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

#[tokio::main]
async fn main() {
    let shared_state: SharedState = Arc::new(TokioMutex::new(Vec::new()));

    let app = Router::new()
        .route("/tasks", post(add_task).get(list_tasks))
        .route("/tasks/:id", patch(complete_task).delete(delete_task))
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

