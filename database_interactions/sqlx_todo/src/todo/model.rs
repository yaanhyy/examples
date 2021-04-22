use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, Row, MySqlPool};
use chrono::prelude::*;
// this struct will use to receive user input
#[derive(Serialize, Deserialize)]
pub struct TodoRequest {
    pub description: String,
    pub done: bool,
}

// this struct will be used to represent database record
#[derive(Serialize, FromRow)]
pub struct Todo {
    pub id: i32,
    pub spec: String,
    pub done: i8,
    pub update_at: NaiveDateTime,
}

// implementation of Actix Responder for Todo struct so we can return Todo from action handler
impl Responder for Todo {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

// Implementation for Todo struct, functions for read/write/update and delete todo from database
impl Todo {
    pub async fn find_all(pool: &MySqlPool) -> Result<Vec<Todo>> {
        let mut todos = vec![];
        let recs = sqlx::query!(
            r#"
                SELECT id, spec, done, update_at
                    FROM todos
                ORDER BY id
            "#
        )
        .fetch_all(pool)
        .await?;

        for rec in recs {
            todos.push(Todo {
                id: rec.id,
                spec: rec.spec,
                done: rec.done,
                update_at: rec.update_at
            });
        }

        Ok(todos)
    }

    pub async fn find_by_id(id: i32, pool: &MySqlPool) -> Result<Todo> {
        let rec = sqlx::query!(
            r#"
                    SELECT * FROM todos WHERE id = ?
                "#,
            id
        )
        .fetch_one(&*pool)
        .await?;

        Ok(Todo {
            id: rec.id,
            spec: rec.spec,
            done: rec.done,
            update_at: rec.update_at
        })
    }

    pub async fn create(todo: TodoRequest, pool: &MySqlPool) -> Result<Todo> {
        let mut tx = pool.begin().await?;
        let todo = sqlx::query("INSERT INTO todos (description, done) VALUES ($1, $2) RETURNING id, description, done")
            .bind(&todo.description)
            .bind(todo.done)
            .map(|row: MySqlRow| {
                Todo {
                    id: row.get(0),
                    spec: row.get(1),
                    done: row.get(2),
                    update_at: row.get(3)
                }
            })
            .fetch_one(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(todo)
    }

    pub async fn update(id: i32, todo: TodoRequest, pool: &MySqlPool) -> Result<Todo> {
        let mut tx = pool.begin().await.unwrap();
        let todo = sqlx::query("UPDATE todos SET description = $1, done = $2 WHERE id = $3 RETURNING id, description, done")
            .bind(&todo.description)
            .bind(todo.done)
            .bind(id)
            .map(|row: MySqlRow| {
                Todo {
                    id: row.get(0),
                    spec: row.get(1),
                    done: row.get(2),
                    update_at: row.get(3)
                }
            })
            .fetch_one(&mut tx)
            .await?;

        tx.commit().await.unwrap();
        Ok(todo)
    }

    pub async fn delete(id: i32, pool: &MySqlPool) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let deleted = sqlx::query("DELETE FROM todos WHERE id = $1")
            .bind(id)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(deleted)
    }
}
