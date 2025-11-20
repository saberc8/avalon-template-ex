import { INestApplication, Injectable, OnModuleDestroy, OnModuleInit } from '@nestjs/common';
import './prisma-env';
import { PrismaClient } from '@prisma/client';

/**
 * PrismaService 封装 PrismaClient，统一管理连接生命周期。
 */
@Injectable()
export class PrismaService
  extends PrismaClient
  implements OnModuleInit, OnModuleDestroy
{
  async onModuleInit(): Promise<void> {
    await this.$connect();
  }

  async onModuleDestroy(): Promise<void> {
    await this.$disconnect();
  }

  // 预留给后续需要接入 Nest 生命周期钩子的场景，目前暂不使用。
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async enableShutdownHooks(app: INestApplication): Promise<void> {
    // 在 Prisma v6 中，beforeExit 事件类型定义较为严格，这里保持空实现即可。
  }
}
