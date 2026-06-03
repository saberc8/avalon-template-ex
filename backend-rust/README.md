# Rust Admin Backend

## Default Account

The seed migrations create `admin/admin123` for local development. Change this password outside local development before exposing the service.

## Migration Smoke Checklist

Run migrations against a local PostgreSQL database:

```bash
DATABASE_URL=postgres://postgres:123456@127.0.0.1:5432/nv_admin sqlx migrate run
```

Check core tables:

```sql
select to_regclass('public.sys_user');
select to_regclass('public.sys_role');
select to_regclass('public.sys_menu');
```

Expected result: each query returns its table name.

## API Error Contract

The Rust backend keeps the existing Avalon/Vue-compatible response envelope for API compatibility:

```json
{
  "code": "403",
  "data": null,
  "msg": "没有访问权限，请联系管理员授权",
  "success": false,
  "timestamp": "1780057589045"
}
```

Application errors, including unauthorized and forbidden responses, are returned with HTTP 200 and a non-`200` business `code`. Frontends must check `code` and `success`, not only the HTTP status.

This compatibility rule applies to JSON APIs. File/download endpoints may still use HTTP status for transport-level failures.
