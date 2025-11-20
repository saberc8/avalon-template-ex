from typing import Any, List

from pydantic import BaseModel


class LabelValue(BaseModel):
    """通用下拉/字典项结构。"""

    label: str
    value: Any
    extra: str | None = None


class DeptTreeNode(BaseModel):
    """部门树结构，兼容前端 TreeNodeData。"""

    key: int
    title: str
    disabled: bool
    children: List["DeptTreeNode"] | None = None


class MenuTreeNode(BaseModel):
    """菜单树结构，兼容前端 TreeNodeData。"""

    key: int
    title: str
    disabled: bool
    children: List["MenuTreeNode"] | None = None


DeptTreeNode.update_forward_refs()
MenuTreeNode.update_forward_refs()


