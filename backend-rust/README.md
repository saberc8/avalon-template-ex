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
