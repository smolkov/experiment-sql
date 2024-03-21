use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

use super::pagination::Pagination;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub notes: String,
    pub completed: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateTodo {
    title: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateTodo {
    title: Option<String>,
    notes: Option<String>,
    completed: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct TodoRepository {
    pool: SqlitePool,
}

impl TodoRepository {
    pub fn new(pool: SqlitePool) -> TodoRepository {
        TodoRepository { pool }
    }
    
    // Create new todo
    pub async fn create(&mut self, todo: CreateTodo) -> Result<i64> {
        let id = sqlx::query("INSERT INTO todos ( title ) VALUES ( ?1 )")
            .bind(todo.title)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
        Ok(id)
    }
    /// Get list of todos support pagination.
    pub async fn list(&mut self, pagination: Pagination) -> Result<Vec<Todo>> {
        let todos: Vec<Todo> = sqlx::query_as(
            "SELECT * FROM todos ORDER BY id LIMIT ?1 OFFSET ?2;",
        )
        .bind(pagination.limit.unwrap_or(u32::MAX))
        .bind(pagination.offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await?;
        Ok(todos)
    }
    /// Get todo from id
    pub async fn get(&mut self, id: i64) -> Result<Todo> {
        let todo: Todo = sqlx::query_as("select * from todos where id = ?1 limit 1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(todo)
    }
    /// Update todo
    pub async fn update(&mut self, id: i64, update: UpdateTodo) -> Result<u64> {
        let todo = self.get(id).await?;
        let rows_affected =
            sqlx::query("UPDATE todos SET title = ?2, notes = ?3, completed = ?4 where id = ?1 ")
                .bind(id)
                .bind(update.title.unwrap_or(todo.title))
                .bind(update.notes.unwrap_or(todo.notes))
                .bind(update.completed.unwrap_or(todo.completed))
                .execute(&self.pool)
                .await?
                .rows_affected();
        Ok(rows_affected)
    }
    /// Delete todo id
    pub async fn delete(&mut self, id: i64) -> Result<u64> {
        Ok(sqlx::query("DELETE from todos where id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected())
    }
    /// Cleanup todos table
    pub async fn cleanup(&mut self) -> Result<u64> {
        Ok(sqlx::query("DELETE from todos")
            .execute(&self.pool)
            .await?
            .rows_affected())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use sqlx::sqlite::SqlitePool;
    pub async fn create_table(pool: SqlitePool) -> Result<()> {
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS todos
        (
            id          INTEGER PRIMARY KEY NOT NULL,
            title       TEXT                NOT NULL,
            notes       TEXT                NOT NULL DEFAULT 'note',
            completed   BOOLEAN             NOT NULL DEFAULT 0
        );"#,
        )
        .execute(&pool)
        .await?;
        Ok(())
    }
    async fn create_repo_and_table() -> Result<TodoRepository> {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let repo = TodoRepository::new(pool);
        create_table(repo.pool.clone()).await.unwrap();
        Ok(repo)
    }

    async fn create_todo(repo: &mut TodoRepository, text: &str) -> Result<i64> {
        let create_todo = CreateTodo {
            title: text.to_owned(),
        };
        let id = repo.create(create_todo).await?;
        Ok(id)
    }

    #[tokio::test]
    async fn test_create() {
        let mut repo = create_repo_and_table().await.unwrap();

        let first_id = create_todo(&mut repo, "Test todo 1").await.unwrap();
        let second_id = create_todo(&mut repo, "Test todo 2").await.unwrap();
        let third_id = create_todo(&mut repo, "Test todo 3").await.unwrap();
        assert_eq!(first_id, 1);
        assert_eq!(second_id, 2);
        assert_eq!(third_id, 3);
    }

    #[tokio::test]
    async fn test_get() {
        let mut repo = create_repo_and_table().await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 1").await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 2").await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 3").await.unwrap();

        let todo1 = repo.get(1).await.unwrap();
        let todo2 = repo.get(2).await.unwrap();
        let todo3 = repo.get(3).await.unwrap();

        assert_eq!(todo1.id, 1);
        assert_eq!(todo2.id, 2);
        assert_eq!(todo3.id, 3);
        assert_eq!(todo1.title, String::from("Test todo 1"));
        assert_eq!(todo2.title, String::from("Test todo 2"));
        assert_eq!(todo3.title, String::from("Test todo 3"));
    }

    #[tokio::test]
    async fn test_update() {
        let mut repo = create_repo_and_table().await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 1").await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 2").await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 3").await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 4").await.unwrap();
        let update_text = UpdateTodo {
            title: Some("Update text only".to_owned()),
            notes: None,
            completed: None,
        };

        let update_done = UpdateTodo {
            title: None,
            notes: None,
            completed: Some(true),
        };
        let update_both = UpdateTodo {
            title: Some("Update text and complete status".to_owned()),
            notes: None,
            completed: Some(true),
        };
        let update_all = UpdateTodo {
            title: Some("Update title , note and complete status".to_owned()),
            notes: Some("Some new notes".to_owned()),
            completed: Some(true),
        };

        let update_second_id = repo.update(2, update_text).await.unwrap();
        let update_third_id = repo.update(3, update_done).await.unwrap();
        let update_fourth_id = repo.update(4, update_both).await.unwrap();
        let _ = repo.update(1, update_all).await.unwrap();

        assert_eq!(update_second_id, 1);
        assert_eq!(update_third_id, 1);
        assert_eq!(update_fourth_id, 1);

        let todo1 = repo.get(1).await.unwrap();
        let todo2 = repo.get(2).await.unwrap();
        let todo3 = repo.get(3).await.unwrap();
        let todo4 = repo.get(4).await.unwrap();
        println!("{todo1:?}");
        println!("{todo2:?}");
        println!("{todo3:?}");
        println!("{todo4:?}");
        assert_eq!(todo1.id, 1);
        assert_eq!(todo2.id, 2);
        assert_eq!(todo3.id, 3);
        assert_eq!(todo1.title, String::from("Update title , note and complete status"));
        assert_eq!(todo2.title, String::from("Update text only"));
        assert_eq!(todo3.title, String::from("Test todo 3"));
        assert_eq!(
            todo4.title,
            String::from("Update text and complete status")
        );
        assert_eq!(todo1.notes, String::from("Some new notes"));
        assert!(todo1.completed);
        assert!(!todo2.completed);
        assert!(todo3.completed);
        assert!(todo4.completed);
    }

    #[tokio::test]
    async fn test_list() {
        let mut repo = create_repo_and_table().await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 1").await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 2").await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 3").await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 4").await.unwrap();
        let pagination = Pagination {
            offset: None,
            limit: None,
        };
        let todos = repo.list(pagination).await.unwrap();
        println!("{todos:?}");
        assert_eq!(todos.len(), 4);
        // test offset only
        let todos = repo.list(Pagination::new(Some(1), None)).await.unwrap();
        println!("{todos:?}");
        assert_eq!(todos.len(), 3);
        assert_eq!(todos.get(0).unwrap().id, 2);
        // test limit  only
        let todos = repo.list(Pagination::new(None, Some(3))).await.unwrap();
        println!("{todos:?}");
        assert_eq!(todos.len(), 3);
        assert_eq!(todos.get(0).unwrap().id, 1);
        // test offset and limit
        let todos = repo.list(Pagination::new(Some(1), Some(2))).await.unwrap();
        println!("{todos:?}");
        assert_eq!(todos.len(), 2);
    }
    #[tokio::test]
    async fn test_delete() {
        let mut repo = create_repo_and_table().await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 1").await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 2").await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 3").await.unwrap();
        let _ = create_todo(&mut repo, "Test todo 4").await.unwrap();

        let affected_rows = repo.delete(3).await.unwrap();
        assert_eq!(affected_rows, 1);
        let todos = repo
            .list(Pagination {
                offset: None,
                limit: None,
            })
            .await
            .unwrap();
        println!("{todos:?}");
        assert_eq!(todos.len(), 3);
    }
}
