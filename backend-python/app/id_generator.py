"""
简单的进程内 ID 生成器。

说明：
- 与 Go 版 `internal/infrastructure/id.Next` 行为等价；
- 基于毫秒时间戳递增，保证在单进程内单调不减；
- 适用于当前管理端后端的主键生成场景。
"""

import threading
import time

_lock = threading.Lock()
_last_millis: int = 0


def next_id() -> int:
    """返回下一个全局唯一 ID（进程内单调递增）。"""
    global _last_millis
    with _lock:
        now = int(time.time() * 1000)
        if now <= _last_millis:
            _last_millis += 1
        else:
            _last_millis = now
        return _last_millis

