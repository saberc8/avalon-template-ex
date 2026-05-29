use sqlx::{FromRow, PgPool};

use crate::{
    domain::data_scope::model::{DeptTree, RoleDeptScope},
    shared::error::AppError,
};

#[derive(Debug, Clone)]
pub struct DeptRepository {
    db: PgPool,
}

#[derive(Debug, FromRow)]
struct DeptEdgeRow {
    parent_id: i64,
    id: i64,
}

#[derive(Debug, FromRow)]
struct RoleDeptRow {
    role_id: i64,
    dept_id: i64,
}

impl DeptRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn enabled_dept_tree(&self) -> Result<DeptTree, AppError> {
        let edges = sqlx::query_as::<_, DeptEdgeRow>(
            r#"
SELECT parent_id, id
FROM sys_dept
WHERE status = 1
ORDER BY parent_id ASC, sort ASC, id ASC;
"#,
        )
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(|row| (row.parent_id, row.id));

        Ok(DeptTree::from_edges(edges))
    }

    pub async fn role_dept_scope(&self, role_ids: &[i64]) -> Result<RoleDeptScope, AppError> {
        if role_ids.is_empty() {
            return Ok(RoleDeptScope::default());
        }

        let role_ids = role_ids.to_vec();
        let pairs = sqlx::query_as::<_, RoleDeptRow>(
            r#"
SELECT rd.role_id, rd.dept_id
FROM sys_role_dept AS rd
JOIN sys_dept AS d ON d.id = rd.dept_id
WHERE rd.role_id = ANY($1)
  AND d.status = 1
ORDER BY rd.role_id ASC, d.sort ASC, rd.dept_id ASC;
"#,
        )
        .bind(role_ids)
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(|row| (row.role_id, row.dept_id));

        Ok(RoleDeptScope::from_pairs(pairs))
    }
}
