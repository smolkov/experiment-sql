use serde::{Deserialize, Serialize};



/// Parameters for pagination
///
/// Used to demonstrate handling of query parameters.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Pagination {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl Pagination {
    pub fn new(offset: Option<u32>, limit: Option<u32>) -> Pagination {
        Pagination { offset, limit }
    }
}
