import * as dotenv from 'dotenv';

// 优先从 .env 加载环境变量，兼容项目其它后端的配置方式
dotenv.config();

// 如未显式提供 DATABASE_URL，则根据 DB_* 环境变量拼接，保持与 Go/Python 版本一致
if (!process.env.DATABASE_URL) {
  const host = process.env.DB_HOST ?? '127.0.0.1';
  const port = process.env.DB_PORT ?? '5432';
  const user = process.env.DB_USER ?? 'postgres';
  const password = process.env.DB_PWD ?? '123456';
  const dbName = process.env.DB_NAME ?? 'nv_admin';
  const sslmode = process.env.DB_SSLMODE ?? 'disable';
  process.env.DATABASE_URL = `postgresql://${user}:${encodeURIComponent(
    password,
  )}@${host}:${port}/${dbName}?sslmode=${sslmode}`;
}

