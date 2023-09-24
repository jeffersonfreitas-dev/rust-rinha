use std::{collections::HashMap, sync::Arc};

use axum::{
    routing::{get, post},
    Router, response::IntoResponse, http::StatusCode, extract::{State, Path}, Json,
};
use serde::{Serialize, Deserialize};
use time::{Date, macros::date};
use tokio::sync::RwLock;
use uuid::Uuid;

time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Clone, Serialize)]
pub struct Person {
    pub id: Uuid,
    #[serde(rename = "nome")]
    pub name: String,
    #[serde(rename = "apelido")]
    pub nick: String,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<String>>,
}

#[derive(Clone, Deserialize)]
pub struct PersonRequest {
    #[serde(rename = "nome")]
    pub name: String,
    #[serde(rename = "apelido")]
    pub nick: String,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<String>>,
}


type AppState = Arc<RwLock<HashMap<Uuid, Person>>>;

#[tokio::main]
async fn main() {

    let mut people: HashMap<Uuid, Person> = HashMap::new();
    let person = Person {
        id: Uuid::now_v7(),
        name: "Jefferson".to_string(),
        nick: "Jeff".to_string(),
        birth_date: date!(1984 - 06 - 06),
        stack: Option::Some(vec!["Rust".to_string()])
    };

    println!("{}", &person.id);

    HashMap::insert(&mut people, person.id, person);
    let app_state = Arc::new(RwLock::new(people));

    let app = Router::new()
        .route("/pessoas", get(search_people))
        .route("/pessoas", post(create_person))
        .route("/pessoas/:id", get(find_person))
        .route("/contagem-pessoas", get(count_people))
        .with_state(app_state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn search_people() -> impl IntoResponse {
    (StatusCode::OK, "")
}

async fn find_person(State(people) : State<AppState>, Path(person_id): Path<Uuid>) -> impl IntoResponse {
    match people.read().await.get(&person_id) {
        Some(person) => Ok(Json(person.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_person(State(people): State<AppState>, Json(person_request): Json<PersonRequest>) -> impl IntoResponse {
    let id = Uuid::now_v7();
    let person = Person {
        id, 
        name: person_request.name,
        birth_date: person_request.birth_date,
        nick: person_request.nick,
        stack: person_request.stack,
    };
    people.write().await.insert(id, person.clone());
    (StatusCode::OK, Json(person))

}

async fn count_people(State(people): State<AppState>) -> impl IntoResponse {
    let count = people.read().await.len();
    (StatusCode::OK, Json(count))
}

