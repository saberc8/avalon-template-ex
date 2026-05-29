use std::collections::{BTreeMap, BTreeSet, VecDeque};

use sqlx::{Postgres, QueryBuilder};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataScope {
    All,
    DeptAndChild,
    Dept,
    SelfOnly,
    Custom,
}

impl DataScope {
    pub fn from_code(code: i16) -> Option<Self> {
        match code {
            1 => Some(Self::All),
            2 => Some(Self::DeptAndChild),
            3 => Some(Self::Dept),
            4 => Some(Self::SelfOnly),
            5 => Some(Self::Custom),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataPermissionTarget<'a> {
    pub dept_column: Option<&'a str>,
    pub user_column: Option<&'a str>,
}

impl DataPermissionTarget<'_> {
    pub fn validate(&self) -> Result<(), DataScopeTargetError> {
        if let Some(column) = self.dept_column {
            validate_column_reference(column)?;
        }
        if let Some(column) = self.user_column {
            validate_column_reference(column)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataScopeFilter {
    pub unrestricted: bool,
    pub dept_ids: Vec<i64>,
    pub self_user_id: Option<i64>,
    dept_column: Option<String>,
    user_column: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataScopeSqlClause {
    Where,
    And,
}

impl DataScopeSqlClause {
    fn sql(self) -> &'static str {
        match self {
            Self::Where => " WHERE ",
            Self::And => " AND ",
        }
    }
}

impl DataScopeFilter {
    pub fn unrestricted() -> Self {
        Self {
            unrestricted: true,
            dept_ids: vec![],
            self_user_id: None,
            dept_column: None,
            user_column: None,
        }
    }

    pub fn restricted(
        target: &DataPermissionTarget<'_>,
        dept_ids: impl IntoIterator<Item = i64>,
        self_user_id: Option<i64>,
    ) -> Result<Self, DataScopeTargetError> {
        target.validate()?;
        let mut dept_ids = dept_ids.into_iter().collect::<Vec<_>>();
        dept_ids.sort_unstable();
        dept_ids.dedup();
        if !dept_ids.is_empty() && target.dept_column.is_none() {
            return Err(DataScopeTargetError::MissingDeptColumn);
        }
        if self_user_id.is_some() && target.user_column.is_none() {
            return Err(DataScopeTargetError::MissingUserColumn);
        }

        Ok(Self {
            unrestricted: false,
            dept_ids,
            self_user_id,
            dept_column: target.dept_column.map(ToOwned::to_owned),
            user_column: target.user_column.map(ToOwned::to_owned),
        })
    }

    pub fn is_unrestricted(&self) -> bool {
        self.unrestricted
    }

    pub fn dept_ids(&self) -> &[i64] {
        &self.dept_ids
    }

    pub fn to_debug_sql(&self) -> String {
        if self.unrestricted {
            return String::new();
        }

        self.debug_predicates().join(" OR ").wrap_predicate()
    }

    pub fn append_where_clause<'args>(&self, query: &mut QueryBuilder<'args, Postgres>) {
        self.append_to_query_builder(query, DataScopeSqlClause::Where);
    }

    pub fn append_and_clause<'args>(&self, query: &mut QueryBuilder<'args, Postgres>) {
        self.append_to_query_builder(query, DataScopeSqlClause::And);
    }

    pub fn append_to_query_builder<'args>(
        &self,
        query: &mut QueryBuilder<'args, Postgres>,
        clause: DataScopeSqlClause,
    ) {
        if self.unrestricted {
            return;
        }

        query.push(clause.sql()).push("(");
        self.append_predicate(query);
        query.push(")");
    }

    fn append_predicate<'args>(&self, query: &mut QueryBuilder<'args, Postgres>) {
        let mut has_predicate = false;

        if !self.dept_ids.is_empty() {
            let dept_column = self
                .dept_column
                .as_deref()
                .expect("data scope filter with dept ids must have a dept column");
            query.push(dept_column).push(" IN (");
            let mut separated = query.separated(", ");
            for dept_id in &self.dept_ids {
                separated.push_bind(*dept_id);
            }
            separated.push_unseparated(")");
            has_predicate = true;
        }

        if let Some(user_id) = self.self_user_id {
            if has_predicate {
                query.push(" OR ");
            }
            let user_column = self
                .user_column
                .as_deref()
                .expect("data scope filter with self user id must have a user column");
            query.push(user_column).push(" = ").push_bind(user_id);
            has_predicate = true;
        }

        if !has_predicate {
            query.push("1 = 0");
        }
    }

    fn debug_predicates(&self) -> Vec<String> {
        if self.unrestricted {
            return vec![];
        }

        let mut predicates = Vec::with_capacity(2);
        if !self.dept_ids.is_empty() {
            let dept_column = self
                .dept_column
                .as_deref()
                .expect("data scope filter with dept ids must have a dept column");
            predicates.push(format!("{dept_column} IN ($dept_ids)"));
        }
        if self.self_user_id.is_some() {
            let user_column = self
                .user_column
                .as_deref()
                .expect("data scope filter with self user id must have a user column");
            predicates.push(format!("{user_column} = $user_id"));
        }
        predicates
    }
}

trait DebugSqlPredicate {
    fn wrap_predicate(self) -> String;
}

impl DebugSqlPredicate for String {
    fn wrap_predicate(self) -> String {
        if self.is_empty() {
            "(1 = 0)".to_string()
        } else {
            format!("({self})")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeptTree {
    children_by_parent: BTreeMap<i64, Vec<i64>>,
}

impl DeptTree {
    pub fn from_edges(edges: impl IntoIterator<Item = (i64, i64)>) -> Self {
        let mut grouped = BTreeMap::<i64, BTreeSet<i64>>::new();
        for (parent_id, child_id) in edges {
            if parent_id == child_id {
                continue;
            }
            grouped.entry(parent_id).or_default().insert(child_id);
        }

        let children_by_parent = grouped
            .into_iter()
            .map(|(parent_id, children)| (parent_id, children.into_iter().collect()))
            .collect();

        Self { children_by_parent }
    }

    pub fn descendants_including(&self, dept_id: i64) -> Vec<i64> {
        let mut visited = BTreeSet::from([dept_id]);
        let mut queue = VecDeque::from([dept_id]);

        while let Some(parent_id) = queue.pop_front() {
            let Some(children) = self.children_by_parent.get(&parent_id) else {
                continue;
            };

            for child_id in children {
                if visited.insert(*child_id) {
                    queue.push_back(*child_id);
                }
            }
        }

        visited.into_iter().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RoleDeptScope {
    dept_ids_by_role: BTreeMap<i64, Vec<i64>>,
}

impl RoleDeptScope {
    pub fn from_pairs(pairs: impl IntoIterator<Item = (i64, i64)>) -> Self {
        let mut grouped = BTreeMap::<i64, BTreeSet<i64>>::new();
        for (role_id, dept_id) in pairs {
            grouped.entry(role_id).or_default().insert(dept_id);
        }

        let dept_ids_by_role = grouped
            .into_iter()
            .map(|(role_id, dept_ids)| (role_id, dept_ids.into_iter().collect()))
            .collect();

        Self { dept_ids_by_role }
    }

    pub fn dept_ids_for_role(&self, role_id: i64) -> &[i64] {
        self.dept_ids_by_role
            .get(&role_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

#[derive(Debug, Error)]
pub enum DataScopeTargetError {
    #[error("数据权限列名不能为空")]
    EmptyColumn,
    #[error("数据权限列名非法: {0}")]
    UnsafeColumn(String),
    #[error("当前数据权限目标缺少部门列")]
    MissingDeptColumn,
    #[error("当前数据权限目标缺少用户列")]
    MissingUserColumn,
}

fn validate_column_reference(column: &str) -> Result<(), DataScopeTargetError> {
    if column.is_empty() {
        return Err(DataScopeTargetError::EmptyColumn);
    }
    if !column.split('.').all(is_sql_identifier) {
        return Err(DataScopeTargetError::UnsafeColumn(column.to_string()));
    }
    Ok(())
}

fn is_sql_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|char| char.is_ascii_alphanumeric() || char == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dept_tree_handles_cycles_without_repeating_departments() {
        let tree = DeptTree::from_edges([(10, 11), (11, 10)]);

        assert_eq!(tree.descendants_including(10), vec![10, 11]);
    }

    #[test]
    fn target_rejects_unsafe_column_references() {
        let target = DataPermissionTarget {
            dept_column: Some("dept_id;DELETE"),
            user_column: None,
        };

        assert!(matches!(
            target.validate(),
            Err(DataScopeTargetError::UnsafeColumn(_))
        ));
    }
}
