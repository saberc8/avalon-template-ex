package auth

import (
	"sort"

	rbac "voc-go-backend/internal/domain/rbac"
)

// RouteItem matches the front-end RouteItem type.
type RouteItem struct {
	ID         int64        `json:"id"`
	Title      string       `json:"title"`
	ParentID   int64        `json:"parentId"`
	Type       int16        `json:"type"`
	Path       string       `json:"path"`
	Name       string       `json:"name"`
	Component  string       `json:"component"`
	Redirect   string       `json:"redirect"`
	Icon       string       `json:"icon"`
	IsExternal bool         `json:"isExternal"`
	IsHidden   bool         `json:"isHidden"`
	IsCache    bool         `json:"isCache"`
	Permission string       `json:"permission"`
	Roles      []string     `json:"roles"`
	Sort       int32        `json:"sort"`
	Status     int16        `json:"status"`
	Children   []RouteItem  `json:"children"`
	ActiveMenu string       `json:"activeMenu"`
	AlwaysShow bool         `json:"alwaysShow"`
	Breadcrumb bool         `json:"breadcrumb"`
	ShowInTabs bool         `json:"showInTabs"`
	Affix      bool         `json:"affix"`
}

// BuildRouteTree builds a simple parent/child tree from flat menus.
// It filters out BUTTON type menus (type=3).
func BuildRouteTree(menus []rbac.Menu, roles []string) []RouteItem {
	// Filter out button type menus.
	var list []rbac.Menu
	for _, m := range menus {
		if m.Type == rbac.MenuTypeButton {
			continue
		}
		list = append(list, m)
	}
	if len(list) == 0 {
		return []RouteItem{}
	}

	// Sort by sort ascending, then id.
	sort.Slice(list, func(i, j int) bool {
		if list[i].Sort == list[j].Sort {
			return list[i].ID < list[j].ID
		}
		return list[i].Sort < list[j].Sort
	})

	// Build id -> RouteItem map.
	nodeMap := make(map[int64]*RouteItem, len(list))
	var roots []RouteItem
	for _, m := range list {
		// Use a fresh composite literal so each map entry has its own stable pointer.
		nodeMap[m.ID] = &RouteItem{
			ID:         m.ID,
			Title:      m.Title,
			ParentID:   m.ParentID,
			Type:       int16(m.Type),
			Path:       m.Path,
			Name:       m.Name,
			Component:  m.Component,
			Redirect:   m.Redirect,
			Icon:       m.Icon,
			IsExternal: m.IsExternal,
			IsHidden:   m.IsHidden,
			IsCache:    m.IsCache,
			Permission: m.Permission,
			Roles:      roles,
			Sort:       m.Sort,
			Status:     m.Status,
			Children:   []RouteItem{},
			// The following flags are not stored in DB; defaults are fine for now.
			ActiveMenu: "",
			AlwaysShow: false,
			Breadcrumb: true,
			ShowInTabs: true,
			Affix:      false,
		}
	}

	// Second pass to build parent-child relationships.
	// First, wire up children using pointers only so the tree structure
	// is complete regardless of map iteration order.
	for _, item := range nodeMap {
		if item.ParentID == 0 {
			continue
		}
		if parent, ok := nodeMap[item.ParentID]; ok {
			parent.Children = append(parent.Children, *item)
		}
	}
	// Then, collect roots as value copies after children are attached.
	for _, item := range nodeMap {
		if item.ParentID == 0 {
			roots = append(roots, *item)
		} else if _, ok := nodeMap[item.ParentID]; !ok {
			// Orphan node: treat as root.
			roots = append(roots, *item)
		}
	}

	// Sort children by sort/id for each node.
	var sortChildren func(nodes []RouteItem)
	sortChildren = func(nodes []RouteItem) {
		for i := range nodes {
			if len(nodes[i].Children) > 0 {
				sort.Slice(nodes[i].Children, func(a, b int) bool {
					if nodes[i].Children[a].Sort == nodes[i].Children[b].Sort {
						return nodes[i].Children[a].ID < nodes[i].Children[b].ID
					}
					return nodes[i].Children[a].Sort < nodes[i].Children[b].Sort
				})
				sortChildren(nodes[i].Children)
			}
		}
	}
	sortChildren(roots)

	return roots
}
