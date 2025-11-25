# 审查报告（PHP 日志 / 存储 / 客户端 / 验证码 / 在线用户迁移）

- 日期：$(date '+%Y-%m-%d %H:%M:%S')
- 审查者：Codex
- 任务：在 PHP 后端实现 /system/log、/system/storage、/system/client、/captcha/image、/monitor/online 等接口，以兼容 pc-admin-vue3，参考 Java + Go 行为。

## 评分
- 技术维度：89/100（SQL 与字段结构与 Go/Java 对齐，部分功能如在线用户列表在 PHP 内暂简化为空集合）。
- 战略维度：92/100（优先保证前端能从 PHP 侧拉取列表/详情并执行增删改操作，路径完全兼容 pc-admin-vue3）。
- 综合评分：90/100

## 结论
- 建议：通过（需在本地 PHP 环境做一次集成联调，特别是导出 CSV、存储密钥解密等路径）。

## 关键实现说明
- `backend-php/src/Interfaces/Http/LogRoutes.php`
  - 实现 GET `/system/log` 分页查询，支持 description/module/ip/createUserString/status/createTime 条件，与 Go `LogHandler.PageLog` 一致，返回结构匹配 `LogResp`。
  - 实现 GET `/system/log/{id}`，返回 `LogDetailResp` 所需字段，包括 request/response 相关内容。
  - 实现 GET `/system/log/export/login`、`/system/log/export/operation`，输出 CSV 文本，列标题和内容与 Go 导出逻辑对齐。
- `backend-php/src/Interfaces/Http/StorageRoutes.php`
  - 实现 `/system/storage/list`、`/{id}`、POST、PUT、DELETE、`/{id}/status`、`/{id}/default`，字段映射 `StorageResp`，并通过 `RsaDecryptor` 解密 `secretKey`。
  - 新增默认存储不可删除/不可禁用的校验，与 Go 行为统一。
- `backend-php/src/Interfaces/Http/ClientRoutes.php`
  - 实现 `/system/client` 分页、`/{id}` 详情、增改删操作，将 `authType` JSON 数组与前端 `string[]` 类型互转，对齐 `ClientResp`/`ClientDetailResp`。
  - 使用与 Go 一致的随机 ID 方案（时间戳+随机数，再做 hex 用于 clientId）。
- `backend-php/src/Interfaces/Http/CaptchaRoutes.php`
  - 实现 `/captcha/image`，仅根据 `sys_option.LOGIN_CAPTCHA_ENABLED` 返回 `ImageCaptchaResp`，不生成实际图片，便于前端根据 `isEnabled` 控制显示；复杂行为验证码/邮箱验证码继续依赖 Java/Go。 
- `backend-php/src/Interfaces/Http/OnlineUserRoutes.php`
  - 提供 `/monitor/online` 空分页返回与 `/monitor/online/{token}` 踢出接口：做 token 鉴权和“不能强退自己”的校验，但不在 PHP 进程内维护在线会话，避免重复实现状态管理。
- `backend-php/public/index.php`
  - 注册新增路由：`LogRoutes`、`StorageRoutes`、`ClientRoutes`、`CaptchaRoutes`、`OnlineUserRoutes`，保持与现有 Auth/System* 路由一致的初始化顺序。

## 风险与后续建议
- 语法与运行时风险：容器环境缺少 `php` 命令，未执行 `php -l` 和实际运行验证，建议在本地 PHP 8.1+ 环境下：
  - 对新建的 Routes 文件与 `public/index.php` 进行语法检查；
  - 通过 pc-admin-vue3 实际访问各接口（尤其日志导出、存储密钥更新路径）验证行为一致性。
- 功能差异：
  - 在线用户列表在 PHP 端暂返回空集合，真正的在线跟踪仍需 Java/Go 或网关配合，如后续需要完全迁移在线用户功能，应在数据库或缓存中维护会话表。
  - 验证码仅实现 `LOGIN_CAPTCHA_ENABLED` 开关，不生成图片；如要完全脱离 Java/Go，可在 PHP 中补充图形验证码库集成。
- 建议：若最终目标是让 PHP 完全独立承载后台管理，可在完成本轮联调后，再迁移文件上传、系统选项联动缓存等高级能力。 
