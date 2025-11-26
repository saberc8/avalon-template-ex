from __future__ import annotations

import hashlib
import os
from datetime import datetime
from typing import Any, Dict, List, Optional

from fastapi import APIRouter, Body, File, Header, Query, UploadFile
from pydantic import BaseModel, Field

from ..api_response import fail, ok
from ..config import get_settings
from ..db import get_db_cursor
from ..id_generator import next_id
from ..security.jwt_token import TokenService

router = APIRouter()


def _get_token_service() -> TokenService:
    """构造 JWT 服务实例。"""
    s = get_settings()
    return TokenService(secret=s.jwt_secret, ttl_seconds=s.jwt_ttl_hours * 3600)


def _current_user_id(authorization: str | None) -> Optional[int]:
    """从 Authorization 头解析当前登录用户 ID，未授权返回 None。"""
    if not authorization:
        return None
    token_svc = _get_token_service()
    claims = token_svc.parse(authorization)
    if not claims:
        return None
    return claims.user_id


def _file_base_url_prefix() -> str:
    """构造本地文件访问前缀，对齐 Go/Java 的 FILE_BASE_URL 配置。"""
    prefix = os.getenv("FILE_BASE_URL", "/file").strip() or "/file"
    if not prefix.startswith("/"):
        prefix = "/" + prefix
    return prefix.rstrip("/")


def _build_local_file_url(path: str) -> str:
    """根据相对路径构造本地访问 URL。"""
    if not path:
        return ""
    if not path.startswith("/"):
        path = "/" + path
    return _file_base_url_prefix() + path


class FileUploadResp(BaseModel):
    """文件上传响应结构。"""

    id: str
    url: str
    thUrl: str
    metadata: Dict[str, Any]


class FileItem(BaseModel):
    """文件列表项结构，兼容前端 FileItem。"""

    id: int
    name: str
    originalName: str
    size: Optional[int] = None
    parentPath: str
    path: str
    extension: str
    contentType: str
    type: int
    sha256: str
    metadata: str
    thumbnailName: str
    thumbnailSize: Optional[int] = None
    thumbnailMetadata: str
    storageId: int
    createTime: str
    createUserString: str
    updateTime: str | None = None
    updateUserString: str
    storageName: str
    url: str
    thumbnailURL: str


class FileDirCalcSizeResp(BaseModel):
    """文件夹大小计算响应。"""

    size: int


class FileStatisticsItem(BaseModel):
    """分类型文件统计。"""

    type: int
    number: int
    size: int


class FileStatisticsResp(BaseModel):
    """文件统计汇总响应。"""

    size: int = 0
    number: int = 0
    data: List[FileStatisticsItem] = Field(default_factory=list)


class IdsRequest(BaseModel):
    """批量 ID 请求体。"""

    ids: List[int]


class PageResult(BaseModel):
    """分页结果泛型容器。"""

    list: List[FileItem]
    total: int


def _format_time(dt: datetime | None) -> str:
    if not dt:
        return ""
    return dt.strftime("%Y-%m-%d %H:%M:%S")


def _normalize_parent_path(parent_path: str | None) -> str:
    """归一化父目录路径，保证以 / 开头，不以 / 结尾（根目录除外）。"""
    p = (parent_path or "").strip()
    if not p or p == "/":
        return "/"
    if not p.startswith("/"):
        p = "/" + p
    # 去掉末尾多余斜杠
    while len(p) > 1 and p.endswith("/"):
        p = p[:-1]
    return p


def _detect_file_type(ext: str, content_type: str) -> int:
    """
    简单文件类型识别，保持与 Go/Java 枚举语义一致：
    0=目录；1=其他；2=图片；3=文档；4=视频；5=音频。
    """
    e = ext.lower().strip(".")
    ct = (content_type or "").lower()
    if not e and not ct:
        return 1

    image_ext = {"png", "jpg", "jpeg", "gif", "webp", "bmp", "svg"}
    doc_ext = {"pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "md"}
    video_ext = {"mp4", "avi", "mov", "mkv", "flv"}
    audio_ext = {"mp3", "wav", "aac", "flac", "ogg"}

    if e in image_ext or ct.startswith("image/"):
        return 2
    if e in doc_ext or ct in {"application/pdf", "text/plain"}:
        return 3
    if e in video_ext or ct.startswith("video/"):
        return 4
    if e in audio_ext or ct.startswith("audio/"):
        return 5
    return 1


def _get_default_storage() -> dict | None:
    """
    查询默认存储配置，参考 Go getDefaultStorage：
    - 优先使用 sys_storage 中 is_default=true 的记录；
    - 如不存在，退回单一本地存储，bucket=./data/file。
    """
    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT id,
       name,
       code,
       type,
       COALESCE(access_key, '')      AS access_key,
       COALESCE(secret_key, '')      AS secret_key,
       COALESCE(endpoint, '')        AS endpoint,
       COALESCE(bucket_name, '')     AS bucket_name,
       COALESCE(domain, '')          AS domain,
       COALESCE(region, '')          AS region,
       COALESCE(is_default, FALSE)   AS is_default,
       COALESCE(status, 1)           AS status
FROM sys_storage
WHERE is_default = TRUE
LIMIT 1;
"""
        )
        row = cur.fetchone()

    if not row:
        # 退回本地存储配置
        return {
            "id": 1,
            "name": "本地存储",
            "code": "local",
            "type": 1,
            "bucket_name": "./data/file",
            "domain": "",
        }

    return row


def _build_storage_file_url(storage: dict | None, full_path: str) -> str:
    """根据存储配置构建访问 URL，对象存储优先使用 domain，本地退回 /file。"""
    if not storage:
        return _build_local_file_url(full_path)
    t = int(storage.get("type") or 1)
    if t == 2:
        domain = (storage.get("domain") or "").strip()
        if not domain:
            return _build_local_file_url(full_path)
        domain = domain.rstrip("/")
        key = full_path.lstrip("/")
        return domain + "/" + key
    return _build_local_file_url(full_path)


def _ensure_local_dir(root: str, parent_path: str) -> str:
    """确保本地存储目录存在，返回最终文件绝对路径目录。"""
    # root 形如 ./data/file
    root_dir = root or "./data/file"
    rel_parent = parent_path.lstrip("/")
    abs_dir = os.path.join(root_dir, rel_parent)
    os.makedirs(abs_dir, exist_ok=True)
    return abs_dir


@router.post("/system/file/upload")
@router.post("/common/file")
def upload_file(
    file: UploadFile = File(...),
    parentPath: str | None = None,
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """上传文件，兼容 /system/file/upload 与 /common/file。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    if not file:
        return fail("400", "文件不能为空")

    parent = _normalize_parent_path(parentPath)
    ext = ""
    original_name = file.filename or ""
    if "." in original_name:
        ext = original_name.rsplit(".", 1)[1].lower()

    storage = _get_default_storage()
    if not storage:
        return fail("500", "获取存储配置失败")

    bucket = storage.get("bucket_name") or "./data/file"
    new_id = next_id()
    stored_name = f"{new_id}.{ext}" if ext else str(new_id)

    # 保存到本地文件系统，并计算 SHA256 与大小。
    abs_dir = _ensure_local_dir(bucket, parent)
    abs_path = os.path.join(abs_dir, stored_name)

    sha = hashlib.sha256()
    size = 0
    try:
        data = file.file.read()
        if not data:
            return fail("400", "文件不能为空")
        sha.update(data)
        size = len(data)
        with open(abs_path, "wb") as f:
            f.write(data)
    except Exception:
        return fail("500", "保存文件失败")

    full_path = parent if parent != "/" else ""
    full_path = f"{full_path}/{stored_name}" if full_path else f"/{stored_name}"
    content_type = file.content_type or ""
    file_type = _detect_file_type(ext, content_type)
    now = datetime.utcnow()

    with get_db_cursor() as cur:
        try:
            cur.execute(
                """
INSERT INTO sys_file (
    id, name, original_name, size, parent_path, path, extension, content_type,
    type, sha256, metadata, thumbnail_name, thumbnail_size, thumbnail_metadata,
    storage_id, create_user, create_time
) VALUES (
    %s, %s, %s, %s, %s, %s, %s, %s,
    %s, %s, %s, %s, %s, %s,
    %s, %s, %s
);
""",
                (
                    new_id,
                    stored_name,
                    original_name,
                    size,
                    parent,
                    full_path,
                    ext,
                    content_type,
                    file_type,
                    sha.hexdigest(),
                    "",
                    "",
                    None,
                    "",
                    int(storage.get("id") or 1),
                    current_uid,
                    now,
                ),
            )
        except Exception:
            return fail("500", "保存文件记录失败")

    url = _build_storage_file_url(storage, full_path)
    resp = FileUploadResp(
        id=str(new_id),
        url=url,
        thUrl=url,
        metadata={},
    )
    return ok(resp.dict())


@router.get("/system/file")
def list_file(
    originalName: str | None = Query(default=None),
    type: str | None = Query(default=None),
    parentPath: str | None = Query(default=None),
    page: int = 1,
    size: int = 30,
):
    """分页查询文件列表：GET /system/file。"""
    if page <= 0:
        page = 1
    if size <= 0:
        size = 30

    where_clauses: List[str] = ["1=1"]
    params: List[Any] = []

    name_kw = (originalName or "").strip()
    if name_kw:
        where_clauses.append("f.original_name ILIKE %s")
        params.append(f"%{name_kw}%")

    type_str = (type or "").strip()
    if type_str and type_str != "0":
        try:
            t = int(type_str)
            if t > 0:
                where_clauses.append("f.type = %s")
                params.append(t)
        except ValueError:
            pass

    parent = (parentPath or "").strip()
    if parent:
        where_clauses.append("f.parent_path = %s")
        params.append(_normalize_parent_path(parent))

    where_sql = " WHERE " + " AND ".join(where_clauses)

    with get_db_cursor() as cur:
        cur.execute(f"SELECT COUNT(*) AS cnt FROM sys_file AS f{where_sql}", params)
        row = cur.fetchone()
        total = int(row["cnt"]) if row and row["cnt"] is not None else 0
        if total == 0:
            return ok(PageResult(list=[], total=0).dict())

        offset = (page - 1) * size
        params_with_limit = params + [size, offset]
        cur.execute(
            f"""
SELECT f.id,
       f.name,
       f.original_name,
       f.size,
       f.parent_path,
       f.path,
       COALESCE(f.extension, '')           AS extension,
       COALESCE(f.content_type, '')        AS content_type,
       f.type,
       COALESCE(f.sha256, '')              AS sha256,
       COALESCE(f.metadata, '')            AS metadata,
       COALESCE(f.thumbnail_name, '')      AS thumbnail_name,
       f.thumbnail_size,
       COALESCE(f.thumbnail_metadata, '')  AS thumbnail_metadata,
       f.storage_id,
       f.create_time,
       COALESCE(cu.nickname, '')           AS create_user_string,
       f.update_time,
       COALESCE(uu.nickname, '')           AS update_user_string
FROM sys_file AS f
LEFT JOIN sys_user AS cu ON cu.id = f.create_user
LEFT JOIN sys_user AS uu ON uu.id = f.update_user
{where_sql}
ORDER BY f.type ASC, f.update_time DESC NULLS LAST, f.id DESC
LIMIT %s OFFSET %s;
""",
            params_with_limit,
        )
        rows = cur.fetchall()

    items: List[FileItem] = []
    for r in rows:
        size_val = r.get("size")
        thumb_size_val = r.get("thumbnail_size")
        create_time = r.get("create_time")
        update_time = r.get("update_time")
        storage_id = int(r.get("storage_id") or 0)

        storage = None
        storage_name = "本地存储"
        if storage_id > 0:
            with get_db_cursor() as cur:
                cur.execute(
                    """
SELECT id, name, type, bucket_name, domain
FROM sys_storage
WHERE id = %s
LIMIT 1;
""",
                    (storage_id,),
                )
                storage = cur.fetchone()
        if storage:
            storage_name = storage.get("name") or "本地存储"

        url = _build_storage_file_url(storage, r["path"])
        thumb_url = url
        thumb_name = r.get("thumbnail_name") or ""
        if thumb_name:
            parent = r.get("parent_path") or "/"
            parent_norm = parent if parent != "/" else ""
            thumb_path = f"{parent_norm}/{thumb_name}" if parent_norm else f"/{thumb_name}"
            thumb_url = _build_storage_file_url(storage, thumb_path)

        item = FileItem(
            id=int(r["id"]),
            name=r["name"],
            originalName=r["original_name"],
            size=int(size_val) if size_val is not None else None,
            parentPath=r["parent_path"],
            path=r["path"],
            extension=r["extension"],
            contentType=r["content_type"],
            type=int(r["type"]),
            sha256=r["sha256"],
            metadata=r["metadata"],
            thumbnailName=thumb_name,
            thumbnailSize=int(thumb_size_val) if thumb_size_val is not None else None,
            thumbnailMetadata=r["thumbnail_metadata"],
            storageId=storage_id,
            createTime=_format_time(create_time),
            createUserString=r["create_user_string"],
            updateTime=_format_time(update_time) if update_time else None,
            updateUserString=r["update_user_string"],
            storageName=storage_name,
            url=url,
            thumbnailURL=thumb_url,
        )
        items.append(item)

    return ok(PageResult(list=items, total=total).dict())


@router.post("/system/file/dir")
def create_dir(
    body: Dict[str, Any] = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """创建文件夹：POST /system/file/dir。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    parent_path = _normalize_parent_path(str(body.get("parentPath") or ""))
    original_name = str(body.get("originalName") or "").strip()
    if not original_name:
        return fail("400", "名称不能为空")

    with get_db_cursor() as cur:
        # 检查同级目录下是否存在同名文件夹
        cur.execute(
            """
SELECT 1
FROM sys_file
WHERE parent_path = %s AND name = %s AND type = 0
LIMIT 1;
""",
            (parent_path, original_name),
        )
        exists = cur.fetchone()
        if exists:
            return fail("400", "文件夹已存在")

        now = datetime.utcnow()
        dir_id = next_id()
        if parent_path == "/":
            path = f"/{original_name}"
        else:
            path = f"{parent_path}/{original_name}"

        try:
            cur.execute(
                """
INSERT INTO sys_file (
    id, name, original_name, size, parent_path, path, extension, content_type,
    type, sha256, metadata, thumbnail_name, thumbnail_size, thumbnail_metadata,
    storage_id, create_user, create_time
) VALUES (
    %s, %s, %s, NULL, %s, %s, NULL, NULL,
    0, '', '', '', NULL, '',
    1, %s, %s
);
""",
                (dir_id, original_name, original_name, parent_path, path, current_uid, now),
            )
        except Exception:
            return fail("500", "创建文件夹失败")

    return ok(True)


@router.get("/system/file/dir/{dir_id}/size")
def calc_dir_size(dir_id: int):
    """计算文件夹大小：GET /system/file/dir/{id}/size。"""
    if dir_id <= 0:
        return fail("400", "ID 参数不正确")

    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT path, type
FROM sys_file
WHERE id = %s
LIMIT 1;
""",
            (dir_id,),
        )
        row = cur.fetchone()

        if not row:
            return fail("404", "文件夹不存在")
        if int(row["type"]) != 0:
            return fail("400", "ID 不是文件夹，无法计算大小")

        path = row["path"]
        prefix = path.rstrip("/") + "/%"
        cur.execute(
            """
SELECT COALESCE(SUM(size), 0) AS total
FROM sys_file
WHERE type <> 0 AND path LIKE %s;
""",
            (prefix,),
        )
        r2 = cur.fetchone()
        total = int(r2["total"]) if r2 and r2["total"] is not None else 0

    return ok(FileDirCalcSizeResp(size=total).dict())


@router.get("/system/file/statistics")
def file_statistics():
    """文件资源统计：GET /system/file/statistics。"""
    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT type,
       COUNT(1)              AS number,
       COALESCE(SUM(size),0) AS size
FROM sys_file
WHERE type <> 0
GROUP BY type;
"""
        )
        rows = cur.fetchall()

    if not rows:
        return ok(FileStatisticsResp().dict())

    items: List[FileStatisticsItem] = []
    total_size = 0
    total_number = 0
    for r in rows:
        item = FileStatisticsItem(
            type=int(r["type"]),
            number=int(r["number"]),
            size=int(r["size"]),
        )
        total_size += item.size
        total_number += item.number
        items.append(item)

    resp = FileStatisticsResp(size=total_size, number=total_number, data=items)
    return ok(resp.dict())


@router.get("/system/file/check")
def check_file(fileHash: str | None = Query(default=None)):
    """检测文件是否存在：GET /system/file/check?fileHash=...。"""
    hash_val = (fileHash or "").strip()
    if not hash_val:
        return ok(None)

    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT f.id,
       f.name,
       f.original_name,
       f.size,
       f.parent_path,
       f.path,
       COALESCE(f.extension, '')          AS extension,
       COALESCE(f.content_type, '')       AS content_type,
       f.type,
       COALESCE(f.sha256, '')             AS sha256,
       COALESCE(f.metadata, '')           AS metadata,
       COALESCE(f.thumbnail_name, '')     AS thumbnail_name,
       f.thumbnail_size,
       COALESCE(f.thumbnail_metadata, '') AS thumbnail_metadata,
       f.storage_id,
       f.create_time,
       COALESCE(cu.nickname, '')          AS create_user_string,
       f.update_time,
       COALESCE(uu.nickname, '')          AS update_user_string
FROM sys_file AS f
LEFT JOIN sys_user AS cu ON cu.id = f.create_user
LEFT JOIN sys_user AS uu ON uu.id = f.update_user
WHERE f.sha256 = %s
LIMIT 1;
""",
            (hash_val,),
        )
        r = cur.fetchone()

    if not r:
        return ok(None)

    size_val = r.get("size")
    thumb_size_val = r.get("thumbnail_size")
    create_time = r.get("create_time")
    update_time = r.get("update_time")
    storage_id = int(r.get("storage_id") or 0)

    storage = None
    storage_name = "本地存储"
    if storage_id > 0:
        with get_db_cursor() as cur:
            cur.execute(
                """
SELECT id, name, type, bucket_name, domain
FROM sys_storage
WHERE id = %s
LIMIT 1;
""",
                (storage_id,),
            )
            storage = cur.fetchone()
    if storage:
        storage_name = storage.get("name") or "本地存储"

    url = _build_storage_file_url(storage, r["path"])
    thumb_url = url
    thumb_name = r.get("thumbnail_name") or ""
    if thumb_name:
        parent = r.get("parent_path") or "/"
        parent_norm = parent if parent != "/" else ""
        thumb_path = f"{parent_norm}/{thumb_name}" if parent_norm else f"/{thumb_name}"
        thumb_url = _build_storage_file_url(storage, thumb_path)

    item = FileItem(
        id=int(r["id"]),
        name=r["name"],
        originalName=r["original_name"],
        size=int(size_val) if size_val is not None else None,
        parentPath=r["parent_path"],
        path=r["path"],
        extension=r["extension"],
        contentType=r["content_type"],
        type=int(r["type"]),
        sha256=r["sha256"],
        metadata=r["metadata"],
        thumbnailName=thumb_name,
        thumbnailSize=int(thumb_size_val) if thumb_size_val is not None else None,
        thumbnailMetadata=r["thumbnail_metadata"],
        storageId=storage_id,
        createTime=_format_time(create_time),
        createUserString=r["create_user_string"],
        updateTime=_format_time(update_time) if update_time else None,
        updateUserString=r["update_user_string"],
        storageName=storage_name,
        url=url,
        thumbnailURL=thumb_url,
    )
    return ok(item.dict())


@router.put("/system/file/{file_id}")
def update_file(
    file_id: int,
    body: Dict[str, Any] = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """重命名文件：PUT /system/file/{id}。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")
    if file_id <= 0:
        return fail("400", "ID 参数不正确")

    original_name = str(body.get("originalName") or "").strip()
    if not original_name:
        return fail("400", "名称不能为空")

    now = datetime.utcnow()
    with get_db_cursor() as cur:
        try:
            cur.execute(
                """
UPDATE sys_file
   SET original_name = %s,
       update_user   = %s,
       update_time   = %s
 WHERE id            = %s;
""",
                (original_name, current_uid, now, file_id),
            )
        except Exception:
            return fail("500", "重命名失败")

    return ok(True)


@router.delete("/system/file")
def delete_file(
    body: IdsRequest = Body(...),
    authorization: str | None = Header(default=None, alias="Authorization"),
):
    """删除文件：DELETE /system/file。"""
    current_uid = _current_user_id(authorization)
    if not current_uid:
        return fail("401", "未授权，请重新登录")

    ids = body.ids or []
    if not ids:
        return fail("400", "ID 列表不能为空")

    # 先加载将要删除的文件记录，用于后续物理删除。
    file_rows: List[dict] = []
    with get_db_cursor() as cur:
        cur.execute(
            """
SELECT id, name, path, parent_path, type, storage_id
FROM sys_file
WHERE id = ANY(%s::bigint[]);
""",
            (ids,),
        )
        file_rows = cur.fetchall()

    # 校验目录是否为空
    with get_db_cursor() as cur:
        for row in file_rows:
            if int(row["type"]) == 0:
                cur.execute(
                    """
SELECT 1
FROM sys_file
WHERE parent_path = %s
LIMIT 1;
""",
                    (row["path"],),
                )
                child = cur.fetchone()
                if child:
                    return fail("400", f"文件夹 [{row['name']}] 不为空，请先删除文件夹下的内容")

    # 删除数据库记录
    with get_db_cursor() as cur:
        try:
            cur.execute(
                "DELETE FROM sys_file WHERE id = ANY(%s::bigint[]);",
                (ids,),
            )
        except Exception:
            return fail("500", "删除文件失败")

    # 尝试删除本地物理文件（仅针对本地存储）
    for row in file_rows:
        if int(row["type"]) == 0:
            continue
        path = row.get("path") or ""
        storage_id = int(row.get("storage_id") or 0)
        storage = None
        if storage_id > 0:
            with get_db_cursor() as cur:
                cur.execute(
                    """
SELECT type, bucket_name
FROM sys_storage
WHERE id = %s
LIMIT 1;
""",
                    (storage_id,),
                )
                storage = cur.fetchone()
        if storage and int(storage.get("type") or 1) != 1:
            # 对象存储场景：当前 Python 版本不做物理删除，仅依赖后端任务或运维清理。
            continue
        bucket = (storage.get("bucket_name") if storage else None) or "./data/file"
        rel = path.lstrip("/")
        abs_path = os.path.join(bucket, rel)
        try:
            if os.path.isfile(abs_path):
                os.remove(abs_path)
        except Exception:
            # 忽略物理删除失败，避免影响主流程。
            continue

    return ok(True)

