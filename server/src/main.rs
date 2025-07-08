use postgres::Error as PostgresError;
use postgres::{Client, NoTls};
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::SystemTime;

#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)]
struct Task {
    id: Option<i32>,
    title: String,
    description: String,
    completed: bool,
    created_at: Option<String>,
}

fn get_db_url() -> String {
    env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://adel:adel123@database:5432/rustapidb".to_string())
}

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT Found\r\n\r\n";
const INTERNAL_ERROR: &str = "HTTP/1.1 500 INTERNAL ERROR\r\n\r\n";

fn main() {
    if let Err(_) = set_database() {
        println!("Error setting database");
        return;
    }
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    println!("Server listening on port 3000");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_client(stream),
            Err(e) => println!("Unable to connect: {}", e),
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let mut request = String::new();

    match stream.read(&mut buffer) {
        Ok(size) => {
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());
            let (status_line, content) = match &*request {
                r if r.starts_with("OPTIONS") => (OK_RESPONSE.to_string(), "".to_string()),
                r if r.starts_with("POST /api/tasks") => handle_post_request(r),
                r if r.starts_with("GET /api/tasks/") => handle_get_request(r),
                r if r.starts_with("GET /api/tasks") => handle_get_all_request(r),
                r if r.starts_with("PUT /api/tasks/") => handle_put_request(r),
                r if r.starts_with("DELETE /api/tasks/") => handle_delete_request(r),
                _ => (NOT_FOUND.to_string(), "404 not found".to_string()),
            };
            stream
                .write_all(format!("{}{}", status_line, content).as_bytes())
                .unwrap();
        }
        Err(e) => eprintln!("Unable to read stream: {}", e),
    }
}

fn handle_post_request(request: &str) -> (String, String) {
    match (
        get_task_request_body(request),
        Client::connect(&get_db_url(), NoTls),
    ) {
        (Ok(task), Ok(mut client)) => {
            let row = client.query_one(
               "INSERT INTO tasks (title, description, completed, created_at) VALUES ($1, $2, $3, NOW()) RETURNING id, created_at",
               &[&task.title, &task.description, &task.completed],
           ).unwrap();

            let task_id: i32 = row.get(0);
            let created_at: SystemTime = row.get(1);

            match client.query_one(
                "SELECT id, title, description, completed, created_at FROM tasks WHERE id = $1",
                &[&task_id],
            ) {
                Ok(row) => {
                    let created_at_sys: SystemTime = row.get(4);
                    let task = Task {
                        id: Some(row.get(0)),
                        title: row.get(1),
                        description: row.get(2),
                        completed: row.get(3),
                        created_at: Some(format_system_time(created_at_sys)),
                    };
                    (
                        OK_RESPONSE.to_string(),
                        serde_json::to_string(&task).unwrap(),
                    )
                }
                Err(_) => (
                    INTERNAL_ERROR.to_string(),
                    "Failed to retrieve created task".to_string(),
                ),
            }
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

fn handle_get_request(request: &str) -> (String, String) {
    match (
        get_id(&request).parse::<i32>(),
        Client::connect(&get_db_url(), NoTls),
    ) {
        (Ok(id), Ok(mut client)) => {
            match client.query_one("SELECT * FROM tasks WHERE id = $1", &[&id]) {
                Ok(row) => {
                    let created_at: SystemTime = row.get(4);
                    let task = Task {
                        id: row.get(0),
                        title: row.get(1),
                        description: row.get(2),
                        completed: row.get(3),
                        created_at: Some(format_system_time(created_at)),
                    };
                    (
                        OK_RESPONSE.to_string(),
                        serde_json::to_string(&task).unwrap(),
                    )
                }
                _ => (NOT_FOUND.to_string(), "Task not found".to_string()),
            }
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

fn handle_get_all_request(_request: &str) -> (String, String) {
    match Client::connect(&get_db_url(), NoTls) {
        Ok(mut client) => {
            let mut tasks = Vec::new();
            for row in client.query("SELECT id, title, description, completed, created_at FROM tasks ORDER BY created_at DESC", &[]).unwrap() {
               let created_at: SystemTime = row.get(4);
               tasks.push(Task {
                   id: row.get(0),
                   title: row.get(1),
                   description: row.get(2),
                   completed: row.get(3),
                   created_at: Some(format_system_time(created_at)),
               });
           }
            (
                OK_RESPONSE.to_string(),
                serde_json::to_string(&tasks).unwrap(),
            )
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

fn handle_put_request(request: &str) -> (String, String) {
    match (
        get_id(&request).parse::<i32>(),
        get_task_request_body(&request),
        Client::connect(&get_db_url(), NoTls),
    ) {
        (Ok(id), Ok(task), Ok(mut client)) => {
            let rows_affected = client
                .execute(
                    "UPDATE tasks SET title = $1, description = $2, completed = $3 WHERE id = $4",
                    &[&task.title, &task.description, &task.completed, &id],
                )
                .unwrap();
            if rows_affected == 0 {
                return (NOT_FOUND.to_string(), "Task not found".to_string());
            }
            (OK_RESPONSE.to_string(), "Task updated".to_string())
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

fn handle_delete_request(request: &str) -> (String, String) {
    match (
        get_id(&request).parse::<i32>(),
        Client::connect(&get_db_url(), NoTls),
    ) {
        (Ok(id), Ok(mut client)) => {
            let rows_affected = client
                .execute("DELETE FROM tasks WHERE id = $1", &[&id])
                .unwrap();
            if rows_affected == 0 {
                return (NOT_FOUND.to_string(), "Task not found".to_string());
            }
            (OK_RESPONSE.to_string(), "Task deleted".to_string())
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

fn set_database() -> Result<(), PostgresError> {
    let mut client = Client::connect(&get_db_url(), NoTls)?;
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS tasks (
           id SERIAL PRIMARY KEY,
           title VARCHAR NOT NULL,
           description TEXT NOT NULL,
           completed BOOLEAN NOT NULL DEFAULT FALSE,
           created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
       )",
    )?;
    Ok(())
}

fn get_id(request: &str) -> &str {
    request
        .split("/")
        .nth(3)
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or_default()
}

fn get_task_request_body(request: &str) -> Result<Task, serde_json::Error> {
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}

fn format_system_time(time: SystemTime) -> String {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let seconds = duration.as_secs();
            let nanos = duration.subsec_nanos();
            format!(
                "{}T{:02}:{:02}:{:02}.{:03}Z",
                chrono::DateTime::from_timestamp(seconds as i64, nanos)
                    .unwrap()
                    .format("%Y-%m-%d"),
                (seconds % 86400) / 3600,
                (seconds % 3600) / 60,
                seconds % 60,
                nanos / 1_000_000
            )
        }
        Err(_) => "1970-01-01T00:00:00.000Z".to_string(),
    }
}
