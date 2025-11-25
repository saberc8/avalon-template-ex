# 审查报告（backend-node 在线用户接口迁移）

- 日期：2025-11-24 00:00:00
- 审查者：Codex
- 任务：在 Node 后端（NestJS + Prisma）中实现与 Java/Go 版本等价的在线用户接口 `/monitor/online`，并对接现有认证登录/登出流程，保证与 pc-admin 前端协议兼容。

## 评分
- 技术维度：88/100
  - 在线会话状态采用进程内 `Map` 维护，结构对齐 Go 版 `OnlineSession`，逻辑简单清晰。
  - 登录入口与登出接口均接入在线存储，支持按昵称与登录时间范围分页查询以及“不能强退自己”等校验。
  - 目前仍依赖 TypeScript 编译配置与部分既有模块修复后，才能完成一次完整的 `npm run build`，存在一定环境依赖风险。
- 战略维度：93/100
  - 路径与返回结构完全复用 Java/Go 约定，前端在线用户页面可在切换后端实现时保持无感。
  - 采用内存态实现在线会话，有利于在不改动数据库结构的前提下快速完成多语言后端的一致化。
- 综合评分：90/100

## 结论
- 建议：通过（前提是本地环境修复 backend-node 既有 TypeScript 配置与相关模块类型问题后，再进行一次集成级联调）。

## 关键实现说明
- 在线存储服务：
  - `backend-node/src/modules/auth/online.store.ts`
    - 定义 `OnlineSession` 与 `OnlineUserResp` 结构，与 Go 版 `OnlineSession` 和 pc-admin `OnlineUserResp` 对齐。
    - 提供：
      - `recordLogin`：在登录成功后写入一条在线会话记录，记录用户 ID、用户名、昵称、客户端 ID、IP、浏览器 UA 等信息，登录时间与最后活跃时间均设置为当前时间。
      - `removeByToken`：按 token 删除会话，用于登出与强退逻辑。
      - `list`：支持基于昵称关键字与登录时间区间过滤在线列表，并按登录时间倒序分页返回。
- 在线用户控制器：
  - `backend-node/src/modules/auth/online.controller.ts`
    - `GET /monitor/online`：
      - 接收 `page/size/nickname/loginTime` 查询参数，其中 `loginTime` 支持前端传入的 `['YYYY-MM-DD HH:mm:ss', 'YYYY-MM-DD HH:mm:ss']` 数组形式。
      - 将查询条件转换为 `OnlineStoreService.list` 参数，返回 `PageResult<OnlineUserResp>`，结构与 Go 版 `PageOnlineUser` 一致。
    - `DELETE /monitor/online/{token}`：
      - 校验 token 非空，并从 `Authorization` 头中解析当前请求 token，禁止“强退自己”。
      - 使用 `TokenService.parse` 校验当前 token 合法性，非法或缺失时返回业务码 `401`。
      - 调用 `OnlineStoreService.removeByToken` 移除目标会话并返回 `ok(true)`。
- 认证流程集成：
  - `backend-node/src/modules/auth/dto/login.dto.ts`：
    - 将 `LoginResp` 扩展为 `{ token,userId,username,nickname }`，仅用于服务端内部传递用户基础信息。
  - `backend-node/src/modules/auth/auth.service.ts`：
    - 在登录成功后返回扩展后的 `LoginResp`，包含用户 ID、用户名与昵称，供控制器记录在线会话使用。
  - `backend-node/src/modules/auth/auth.controller.ts`：
    - 登录接口：
      - 注入 `OnlineStoreService` 并新增 `@Req()` 参数，从 `X-Forwarded-For` 或 `req.ip` 获取 IP，从 `User-Agent` 头获取浏览器信息。
      - 调用 `onlineStore.recordLogin` 写入在线记录后，通过 `ok({ token: resp.token })` 仅向前端暴露 `token` 字段，保持原有协议兼容。
    - 登出接口：
      - 解析 `Authorization` 头获取原始 token，先用 `TokenService.parse` 做基本鉴权，再调用 `onlineStore.removeByToken` 清理当前进程内在线记录。
- 模块装配：
  - `backend-node/src/modules/auth/auth.module.ts`
    - 将 `OnlineStoreService` 注册为 provider，并新增 `OnlineUserController` 至 controllers，复用既有 `TokenService` 与认证模块，无需额外创建新模块或改动应用根模块装配。

## 风险与后续建议
- 当前风险：
  - `npm run build` 在 Codex 环境下因既有 TypeScript 配置与 `system-option/system-user/id` 等模块类型问题失败，在线用户实现本身尚未在完整构建结果中经过验证。
  - 在线会话存储仅存在于单个 Node 进程内，服务重启后在线列表会被清空，且强退仅从内存中移除记录，并不会立即使 JWT 本身失效。
- 建议：
  - 在本地修复 backend-node 中与 Prisma 事务、`PasswordService.hash`、`Express` 类型与 BigInt 目标版本相关的 TypeScript 问题，确保 `npm run build` 能顺利通过。
  - 启动 Node 后端并配合 pc-admin 前端，完成以下冒烟验证：
    - 登录后台后访问“系统监控 → 在线用户”，确认列表中出现当前登录用户记录，并可按昵称与时间范围筛选。
    - 使用在线用户页面对非当前会话执行强退操作，确认 `/monitor/online/{token}` 返回成功并刷新列表；对当前 token 强退时返回“不能强退自己”提示。
    - 通过前端执行登出操作，确认调用 `POST /auth/logout` 后在线列表中对应会话被移除。
  - 若后续需要在分布式场景下维持统一的在线用户视图，可在现有内存实现基础上，引入基于数据库或缓存（如 Redis）的集中式在线会话存储，并在各语言后端中保持统一设计。 

