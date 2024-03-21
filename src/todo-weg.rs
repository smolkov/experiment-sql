use std::collections::HashMap;
use std::sync::atomic::{ AtomicUsize,Ordering};
use sqlx::sqlite::SqlitePool;
use anyhow::Result;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TodoItem {
    pub title: String,
    pub notes: String,
    pub assigned: String,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateTodoItem {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub assigned: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdentifiableTodoItem {
    pub id: usize,

    #[serde(flatten)]
    pub todo: TodoItem,
}

impl IdentifiableTodoItem {
    pub fn new(id: usize, todo: TodoItem) -> IdentifiableTodoItem {
        IdentifiableTodoItem { id, todo }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pagination {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

impl Pagination {
    pub fn new(offset: Option<usize>, limit: Option<usize>) -> Pagination {
        Pagination { offset, limit }
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Pagination{
            offset: None,
            limit: None
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TodoStoreError {
    #[error("persistent data store Error")]
    FileAccessError(#[from] std::io::Error),
    #[error("serialization error")]
    SerializationError(#[from] serde_json::Error),
}


#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TodoRepository {
    pool: SqlitePool,
}

impl TodoRepository {
    pub fn new(pool: SqlitePool ) -> TodoRepository {
        TodoRepository{pool}
    }

    fn get_todo(&self, id: usize) -> Result<Option<IdentifiableTodoItem>> {
        let find_query = r#"
        select todos.* 
        from todos 
        where todos.id=$1"#;

        let items = sqlx::query_as::<_, IdentifiableTodoItem>(find_query)
            .bind(id)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| match err {
                sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
                _ => RepositoryError::Unexpected(err.to_string()),
            })?;
        None
    }
}



pub struct TodoStore {
    store: HashMap<usize, IdentifiableTodoItem>,
    id_generator: AtomicUsize,
}

impl TodoStore {
    pub fn from_hashmap(store: HashMap<usize, IdentifiableTodoItem>) -> TodoStore {
        let id_generator = AtomicUsize::new(store.keys().max().map(|v| v + 1).unwrap_or(0));
        TodoStore {
            store,
            id_generator,
        }
    }
	/// Get list of todo
	/// 
	/// Support pagination
    pub fn get_todos(&self, pagination: Pagination) -> Vec<IdentifiableTodoItem> {
        self.store
            .values()
            .skip(pagination.offset.unwrap_or(0))
            .take(pagination.limit.unwrap_or(usize::MAX))
            .cloned()
            .collect()
    }
    /// Get todo item by id
	pub fn get_todo(&self,id: usize) -> Option<&IdentifiableTodoItem> {
		self.store.get(&id)	
	}
	/// Create new todo
	pub fn add_todo(&mut self, todo: TodoItem) -> IdentifiableTodoItem {
		let id = self.id_generator.fetch_add(1, Ordering::Relaxed);
		let new_todo = IdentifiableTodoItem{id,todo};
		self.store.insert(id,new_todo.clone());
		new_todo
	}
	/// Remove todo item by id
	pub fn remove_todo(&mut self, id: usize) -> Option<IdentifiableTodoItem>{
		self.store.remove(&id)
	}

	/// Update todo item by id
	pub fn update_item(&mut self, id: usize,todo: UpdateTodoItem) -> Option<&IdentifiableTodoItem> {
		if let Some(item) = self.store.get_mut(&id) {
			if let Some(title) = todo.title {
				item.todo.title = title;
			}
			if let Some(notes) = todo.notes {
				item.todo.notes = notes;
			}
			if let Some(assigned) = todo.assigned {
				item.todo.assigned = assigned;
			}
			if let Some(completed) = todo.completed {
				item.todo.completed = completed;
			}
			return Some(item);
		}
		None
	}
}


impl Default for TodoStore {
	fn default() -> Self {
		TodoStore::from_hashmap(HashMap::new())
	}
}