package rbac

// MenuType corresponds to sys_menu.type (MenuTypeEnum in Java).
// 1: DIR, 2: MENU, 3: BUTTON.
type MenuType int16

const (
	MenuTypeDir   MenuType = 1
	MenuTypeMenu  MenuType = 2
	MenuTypeButton MenuType = 3
)

// Menu represents a menu / route item (sys_menu).
type Menu struct {
	ID          int64
	ParentID    int64
	Title       string
	Type        MenuType
	Path        string
	Name        string
	Component   string
	Redirect    string
	Icon        string
	IsExternal  bool
	IsCache     bool
	IsHidden    bool
	Permission  string
	Sort        int32
	Status      int16
}

