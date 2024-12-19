use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{post, patch},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct Task {
        id: Uuid,
        description: String,
        completed: bool,
    }

    impl Task {
        fn new(description: String) -> Self {
            Self {
                id: Uuid::new_v4(),
                description,
                completed: false,
            }
        }
    }

    type SharedState = Arc<Mutex<Vec<Task>>>;

    #[derive(Deserialize)]
    struct AddTask {
        description: String,
    }

    async fn add_task(
        State(state): State<SharedState>,
        Json(payload): Json<AddTask>,
    ) -> Json<Task> {
        let mut tasks = state.lock().unwrap();
        let new_task = Task::new(payload.description);
        tasks.push(new_task.clone());
        Json(new_task)
    }

    async fn list_tasks(State(state): State<SharedState>) -> Json<Vec<Task>> {
        let tasks = state.lock().unwrap();
        Json(tasks.clone())
    }

    async fn complete_task(
        State(state): State<SharedState>,
        Path(id): Path<Uuid>,
    ) -> StatusCode {
        let mut tasks = state.lock().unwrap();
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            task.completed = true;
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    }

    async fn delete_task(
        State(state): State<SharedState>,
        Path(id): Path<Uuid>,
    ) -> StatusCode {
        let mut tasks = state.lock().unwrap();
        let len_before = tasks.len();
        tasks.retain(|task| task.id != id);
        if tasks.len() < len_before {
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    }

    let shared_state: SharedState = Arc::new(Mutex::new(Vec::new()));

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

