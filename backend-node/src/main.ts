import 'reflect-metadata';
import * as dotenv from 'dotenv';
import { NestFactory } from '@nestjs/core';
import { AppModule } from './modules/app.module';

// 在应用启动前加载环境变量，兼容 DB_* / AUTH_* 等配置
dotenv.config();

async function bootstrap() {
  const app = await NestFactory.create(AppModule);

  // 简单 CORS 设置，与 Go/Python 版本保持一致（主要用于本地前端调试）
  app.enableCors({
    origin: (origin: string | undefined, callback: (err: Error | null, origin?: any) => void) => {
      if (!origin || origin === 'http://localhost:3000') {
        callback(null, origin);
      } else {
        callback(null, false);
      }
    },
    credentials: true,
    allowedHeaders: 'Content-Type, Authorization, X-Requested-With',
    methods: 'GET,POST,PUT,DELETE,OPTIONS,PATCH',
  });

  const port = process.env.HTTP_PORT || '4398';
  await app.listen(parseInt(port, 10));
}

bootstrap().catch((err) => {
  // 这里直接抛出错误，交由进程管理工具处理
  // eslint-disable-next-line no-console
  console.error('NestJS backend-node 启动失败：', err);
  process.exit(1);
});
