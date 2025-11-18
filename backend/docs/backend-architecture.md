# Backend 架构说明

本项目后端基于 **ContiNew Admin** 框架，采用 Spring Boot 3 + MyBatis Plus + Liquibase 等技术栈，按功能模块化拆分，核心由以下几部分组成：

## 一、整体模块划分

- `continew-webapi`：对外提供 HTTP API 的 Web 层，也是最终打包和部署的启动模块。
  - `config`：全局配置（安全、日志、拦截器、Sa-Token 等）。
  - `controller`：按业务域划分的 REST 控制器，例如：`auth`、`common`、`monitor`、`system` 等。
  - `resources/config`：多环境配置文件（`application.yml`、`application-dev.yml` 等）。
  - `resources/db/changelog`：Liquibase 数据库变更脚本（MySQL / PostgreSQL）。
  - `resources/templates`：邮件、导入等模板文件。
- `continew-module-system`：系统管理领域模块，包含用户、角色、部门、字典、公告、存储配置、短信等业务。
  - `auth`：认证相关业务（登录、权限、Token 等）。
  - `system`：系统管理领域核心代码：
    - `config`：系统配置（文件、短信、邮件等）。
    - `enums`：系统内通用枚举（状态、类型等）。
    - `mapper`：MyBatis Plus Mapper 接口；部分复杂查询会额外在 XML 中编写 SQL。
    - `model`：DTO / VO 模型层，按用途再细分：
      - `entity`：实体类，直接映射数据库表（通常继承 `BaseDO` 或 `BaseCreateDO`）。
      - `query`：列表/分页查询条件对象，配合 `@Query` 注解生成动态查询条件。
      - `req`：创建/修改请求对象（Request），负责入参与校验。
      - `resp`：列表/详情响应对象（Response），用于对外返回数据。
    - `service`：业务接口，通常继承 `BaseService`，对外暴露领域服务能力。
    - `service.impl`：业务实现类，通常继承 `BaseServiceImpl`，封装通用 CRUD 能力及扩展逻辑。
    - `validation` / `util`：系统领域下的校验规则、工具类等。
- `continew-common`：通用基础能力模块。
  - `common.controller.BaseController`：统一的 CRUD 控制器基类。
  - `common.model.entity.BaseDO` / `BaseCreateDO`：统一的实体基类（ID + 审计字段）。
  - `common.model.resp.BaseResp`：统一的响应基类（ID + 审计字段）。
  - 公共常量、枚举、异常、工具类等。
- `continew-extension`、`continew-plugin`：对 Starter、代码生成器、任务调度等扩展，不在本次 Banner CRUD 范围内。

## 二、分层设计

整体采用经典的 **Controller → Service → Mapper → DB** 分层，并在 Service 层叠加通用 CRUD 扩展：

- **Controller 层（Web 层）**
  - 位置：`continew-webapi/src/main/java/top/continew/admin/controller/**`。
  - 依赖：`BaseController` + `@CrudRequestMapping` + `@Tag` / `@Operation` 等 OpenAPI 注解。
  - 通过 `@CrudRequestMapping` 指定：
    - `value`：统一的路由前缀，例如 `/system/notice`。
    - `api`：启用的 CRUD 能力（`Api.PAGE`、`Api.LIST`、`Api.GET`、`Api.CREATE`、`Api.UPDATE`、`Api.DELETE`、`Api.EXPORT` 等）。
  - 通用接口（分页、列表、详情、新增、修改、删除）由 `BaseController` 和 `BaseServiceImpl` 自动实现。
  - 如需额外业务接口，可在 Controller 中自行新增方法，并注入 Service 调用。

- **Service 层（领域服务层）**
  - 接口定义位置：`continew-module-system/src/main/java/top/continew/admin/system/service/**`。
  - 实现类位置：`continew-module-system/src/main/java/top/continew/admin/system/service/impl/**`。
  - 通常模式：
    - Service 接口：`public interface XxxService extends BaseService<XxxResp, XxxDetailResp, XxxQuery, XxxReq> { ... }`
    - Service 实现：`public class XxxServiceImpl extends BaseServiceImpl<XxxMapper, XxxDO, XxxResp, XxxDetailResp, XxxQuery, XxxReq> implements XxxService { ... }`
  - `BaseServiceImpl` 中已经封装了标准的分页、列表、详情、新增、修改、删除等通用逻辑；
    - 通过覆写 `beforeCreate` / `afterCreate` / `beforeUpdate` / `afterUpdate` / `beforeDelete` / `afterDelete` 等模板方法可挂载额外业务校验或副作用逻辑。

- **Mapper 层（持久化层）**
  - 位置：`continew-module-system/src/main/java/top/continew/admin/system/mapper/**`。
  - 模式：`public interface XxxMapper extends BaseMapper<XxxDO> { ... }`
  - 基于 MyBatis Plus 提供的 `BaseMapper` 通用 CRUD 能力，复杂查询可以通过：
    - Mapper XML（位于 `continew-module-system/src/main/resources/mapper`）自定义；
    - 或在 Service 中使用 Lambda Query 构造查询。

- **模型层（DTO / VO / Entity）**
  - Entity：`entity` 包下，映射到特定数据库表，继承 `BaseDO` 或 `BaseCreateDO`，带有统一的审计字段和 ID。
  - Req：`req` 包下，对应新增/修改接口的请求体，结合 `jakarta.validation` 注解进行参数校验。
  - Resp：`resp` 包下，对应列表或详情接口响应体，可以继承 `BaseResp` 复用审计字段。
  - Query：`query` 包下，结合 `@Query` 注解定义过滤条件（精确匹配 / 模糊匹配 / 范围查询等）。

## 三、通用 CRUD 约定

- **路由与权限**
  - Controller 使用 `@CrudRequestMapping("/system/xxx")` 声明基础路由前缀。
  - `BaseController` 在 `preHandle` 中根据路由前缀和接口类型自动计算权限码，例如：
    - `/system/notice` + `Api.PAGE` → 权限码：`system:notice:list`
    - `/system/notice` + `Api.GET` → 权限码：`system:notice:get`
    - `/system/notice` + `Api.CREATE` → 权限码：`system:notice:create`
    - `/system/notice` + `Api.UPDATE` → 权限码：`system:notice:update`
    - `/system/notice` + `Api.DELETE` → 权限码：`system:notice:delete`
  - 前端在菜单 / 按钮上按上述权限码进行控制即可。

- **分页 / 列表**
  - `Api.PAGE`：启用分页查询接口，路径通常为：`GET /system/xxx/page`，入参为 `XxxQuery` + `PageQuery`。
  - `Api.LIST`：启用简单列表接口，路径通常为：`GET /system/xxx/list`。
  - `Api.TREE`：启用树结构接口（如部门、菜单）。

- **参数校验与数据字典**
  - 请求参数通过 `jakarta.validation` + `@Validated` 进行校验。
  - 部分字段（如枚举、状态、字典类型）会绑定数据字典，配合前端展示枚举中文标签。

## 四、按照架构新增业务（以 Banner 为例）

新增一个 Banner 管理功能时，整体步骤如下：

1. **设计数据库表（建议）**
   - 在 `resources/db/changelog/mysql` 中新增 Liquibase 变更脚本，创建 `sys_banner` 表，例如字段：
     - `id`（主键，自增）
     - `title`（标题）
     - `image_url`（图片 URL）
     - `link_url`（跳转链接，可选）
     - `sort`（排序）
     - `status`（启用/禁用）
     - `remark`（备注）
     - `create_user`、`create_time`、`update_user`、`update_time`（由 `BaseDO` 提供）。

2. **在 `continew-module-system` 中按约定新增：**
   - `model/entity/BannerDO`：映射 `sys_banner` 表，继承 `BaseDO`。
   - `model/query/BannerQuery`：查询条件，例如按标题模糊搜索、按状态过滤等。
   - `model/req/BannerReq`：新增/修改请求体，包含标题、图片地址、跳转地址、排序、状态等字段与校验注解。
   - `model/resp/BannerResp`：列表/详情响应对象，继承 `BaseResp`。
   - `mapper/BannerMapper`：继承 `BaseMapper<BannerDO>`，如有复杂查询可扩展方法。
   - `service/BannerService`：继承 `BaseService<BannerResp, BannerResp, BannerQuery, BannerReq>`。
   - `service/impl/BannerServiceImpl`：继承 `BaseServiceImpl<BannerMapper, BannerDO, BannerResp, BannerResp, BannerQuery, BannerReq>`。

3. **在 `continew-webapi` 中按约定新增：**
   - `controller/system/BannerController`：
     - 继承 `BaseController<BannerService, BannerResp, BannerResp, BannerQuery, BannerReq>`。
     - 标注：`@CrudRequestMapping(value = "/system/banner", api = {Api.PAGE, Api.GET, Api.CREATE, Api.UPDATE, Api.DELETE})`。
     - 根据需要补充自定义接口（例如：查询前台展示 Banner 列表等）。

4. **前端接入与权限控制**
   - 前端以 `/system/banner` 为基础路由，调用分页、详情、新增、修改、删除接口。
   - 菜单/按钮权限码遵循：`system:banner:list`、`system:banner:get`、`system:banner:create`、`system:banner:update`、`system:banner:delete`。

下面的 Banner CRUD 实现将严格遵循上述架构与约定。
