use std::collections::HashMap;

use serde::Serialize;

use crate::domain::auth::model::{CurrentUser, ADMIN_ROLE_CODE};

pub const ALL_PERMISSION: &str = "*:*:*";
const LEGACY_WILDCARD_PERMISSION: &str = "*";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionContext {
    pub role_codes: Vec<String>,
    pub permissions: Vec<String>,
}

impl PermissionContext {
    pub fn has(&self, permission: &str) -> bool {
        self.is_admin()
            || self.permissions.iter().any(|existing| {
                existing == ALL_PERMISSION
                    || existing == LEGACY_WILDCARD_PERMISSION
                    || existing == permission
            })
    }

    pub fn is_admin(&self) -> bool {
        self.role_codes
            .iter()
            .any(|role_code| role_code == ADMIN_ROLE_CODE)
    }
}

impl From<&CurrentUser> for PermissionContext {
    fn from(user: &CurrentUser) -> Self {
        Self {
            role_codes: user.roles.iter().map(|role| role.code.clone()).collect(),
            permissions: user.permissions.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuType {
    Dir,
    Menu,
    Button,
}

impl From<i16> for MenuType {
    fn from(value: i16) -> Self {
        match value {
            1 => Self::Dir,
            3 => Self::Button,
            _ => Self::Menu,
        }
    }
}

impl From<MenuType> for i16 {
    fn from(value: MenuType) -> Self {
        match value {
            MenuType::Dir => 1,
            MenuType::Menu => 2,
            MenuType::Button => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    pub id: i64,
    pub parent_id: i64,
    pub title: String,
    pub menu_type: MenuType,
    pub path: String,
    pub name: String,
    pub component: String,
    pub redirect: String,
    pub icon: String,
    pub is_external: bool,
    pub is_cache: bool,
    pub is_hidden: bool,
    pub permission: String,
    pub sort: i32,
    pub status: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteItem {
    pub id: i64,
    pub title: String,
    pub parent_id: i64,
    #[serde(rename = "type")]
    pub menu_type: i16,
    pub path: String,
    pub name: String,
    pub component: String,
    pub redirect: String,
    pub icon: String,
    pub is_external: bool,
    pub is_hidden: bool,
    pub is_cache: bool,
    pub permission: String,
    pub roles: Vec<String>,
    pub sort: i32,
    pub status: i16,
    pub children: Vec<RouteItem>,
    pub active_menu: String,
    pub always_show: bool,
    pub breadcrumb: bool,
    pub show_in_tabs: bool,
    pub affix: bool,
}

pub fn build_route_tree(menus: Vec<Menu>, roles: Vec<String>) -> Vec<RouteItem> {
    let menu_by_id: HashMap<i64, Menu> = menus
        .into_iter()
        .filter(|menu| menu.menu_type != MenuType::Button)
        .map(|menu| (menu.id, menu))
        .collect();

    let mut child_ids: HashMap<i64, Vec<i64>> = HashMap::new();
    for menu in menu_by_id.values() {
        if menu.parent_id != 0 && menu_by_id.contains_key(&menu.parent_id) {
            child_ids.entry(menu.parent_id).or_default().push(menu.id);
        }
    }

    for ids in child_ids.values_mut() {
        sort_menu_ids(ids, &menu_by_id);
    }

    let mut root_ids = menu_by_id
        .values()
        .filter(|menu| menu.parent_id == 0)
        .map(|menu| menu.id)
        .collect::<Vec<_>>();
    sort_menu_ids(&mut root_ids, &menu_by_id);

    root_ids
        .into_iter()
        .map(|id| build_route_item(id, &menu_by_id, &child_ids, &roles))
        .collect()
}

fn build_route_item(
    id: i64,
    menu_by_id: &HashMap<i64, Menu>,
    child_ids: &HashMap<i64, Vec<i64>>,
    roles: &[String],
) -> RouteItem {
    let menu = menu_by_id
        .get(&id)
        .expect("route tree child id must exist in menu map");
    let children = child_ids
        .get(&id)
        .map(|ids| {
            ids.iter()
                .map(|child_id| build_route_item(*child_id, menu_by_id, child_ids, roles))
                .collect()
        })
        .unwrap_or_default();

    RouteItem {
        id: menu.id,
        title: menu.title.clone(),
        parent_id: menu.parent_id,
        menu_type: i16::from(menu.menu_type),
        path: menu.path.clone(),
        name: menu.name.clone(),
        component: menu.component.clone(),
        redirect: menu.redirect.clone(),
        icon: menu.icon.clone(),
        is_external: menu.is_external,
        is_hidden: menu.is_hidden,
        is_cache: menu.is_cache,
        permission: menu.permission.clone(),
        roles: roles.to_vec(),
        sort: menu.sort,
        status: menu.status,
        children,
        active_menu: String::new(),
        always_show: false,
        breadcrumb: true,
        show_in_tabs: true,
        affix: false,
    }
}

fn sort_menu_ids(ids: &mut [i64], menu_by_id: &HashMap<i64, Menu>) {
    ids.sort_by(|left, right| {
        let left_menu = menu_by_id
            .get(left)
            .expect("route tree sort id must exist in menu map");
        let right_menu = menu_by_id
            .get(right)
            .expect("route tree sort id must exist in menu map");

        left_menu
            .sort
            .cmp(&right_menu.sort)
            .then(left_menu.id.cmp(&right_menu.id))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_role_has_all_permissions() {
        let ctx = PermissionContext {
            role_codes: vec!["admin".to_string()],
            permissions: vec![],
        };

        assert!(ctx.has("system:user:delete"));
    }

    #[test]
    fn wildcard_permission_has_all_permissions() {
        let ctx = PermissionContext {
            role_codes: vec!["general".to_string()],
            permissions: vec!["*".to_string()],
        };

        assert!(ctx.has("system:user:delete"));
    }

    #[test]
    fn java_wildcard_permission_has_all_permissions() {
        let ctx = PermissionContext {
            role_codes: vec!["general".to_string()],
            permissions: vec!["*:*:*".to_string()],
        };

        assert!(ctx.has("system:user:delete"));
    }

    #[test]
    fn normal_role_requires_explicit_permission() {
        let ctx = PermissionContext {
            role_codes: vec!["general".to_string()],
            permissions: vec!["system:user:list".to_string()],
        };

        assert!(ctx.has("system:user:list"));
        assert!(!ctx.has("system:user:delete"));
    }

    #[test]
    fn route_tree_excludes_buttons_and_preserves_nested_menu_children() {
        let routes = build_route_tree(
            vec![
                Menu {
                    id: 1,
                    parent_id: 0,
                    title: "系统管理".to_string(),
                    menu_type: MenuType::Dir,
                    path: "/system".to_string(),
                    name: "System".to_string(),
                    component: "Layout".to_string(),
                    redirect: "/system/config".to_string(),
                    icon: "settings".to_string(),
                    is_external: false,
                    is_cache: false,
                    is_hidden: false,
                    permission: "".to_string(),
                    sort: 1,
                    status: 1,
                },
                Menu {
                    id: 2,
                    parent_id: 1,
                    title: "系统配置".to_string(),
                    menu_type: MenuType::Menu,
                    path: "/system/config".to_string(),
                    name: "SystemConfig".to_string(),
                    component: "system/config/index".to_string(),
                    redirect: "".to_string(),
                    icon: "config".to_string(),
                    is_external: false,
                    is_cache: false,
                    is_hidden: false,
                    permission: "".to_string(),
                    sort: 2,
                    status: 1,
                },
                Menu {
                    id: 3,
                    parent_id: 2,
                    title: "网站配置".to_string(),
                    menu_type: MenuType::Menu,
                    path: "/system/config?tab=site".to_string(),
                    name: "SystemSiteConfig".to_string(),
                    component: "system/config/site/index".to_string(),
                    redirect: "".to_string(),
                    icon: "apps".to_string(),
                    is_external: false,
                    is_cache: false,
                    is_hidden: true,
                    permission: "".to_string(),
                    sort: 1,
                    status: 1,
                },
                Menu {
                    id: 4,
                    parent_id: 3,
                    title: "查询".to_string(),
                    menu_type: MenuType::Button,
                    path: "".to_string(),
                    name: "".to_string(),
                    component: "".to_string(),
                    redirect: "".to_string(),
                    icon: "".to_string(),
                    is_external: false,
                    is_cache: false,
                    is_hidden: false,
                    permission: "system:siteConfig:get".to_string(),
                    sort: 1,
                    status: 1,
                },
            ],
            vec!["admin".to_string()],
        );

        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].children.len(), 1);
        assert_eq!(routes[0].children[0].children.len(), 1);
        assert_eq!(routes[0].children[0].children[0].id, 3);
        assert!(routes[0].children[0].children[0].children.is_empty());
    }

    #[test]
    fn route_tree_does_not_promote_orphan_child_when_parent_is_missing() {
        let routes = build_route_tree(
            vec![Menu {
                id: 2,
                parent_id: 1,
                title: "用户管理".to_string(),
                menu_type: MenuType::Menu,
                path: "/system/user".to_string(),
                name: "SystemUser".to_string(),
                component: "system/user/index".to_string(),
                redirect: "".to_string(),
                icon: "user".to_string(),
                is_external: false,
                is_cache: false,
                is_hidden: false,
                permission: "".to_string(),
                sort: 1,
                status: 1,
            }],
            vec!["general".to_string()],
        );

        assert!(routes.is_empty());
    }
}
