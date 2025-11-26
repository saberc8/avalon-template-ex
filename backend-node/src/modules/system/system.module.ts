import { Module } from '@nestjs/common';
import { PrismaService } from '../../shared/prisma/prisma.service';
import { TokenService } from '../auth/jwt/jwt.service';
import { RSADecryptor } from '../auth/security/rsa.service';
import { PasswordService } from '../auth/security/password.service';
import { SystemUserController } from './user/system-user.controller';
import { SystemRoleController } from './role/system-role.controller';
import { SystemMenuController } from './menu/system-menu.controller';
import { SystemDeptController } from './dept/system-dept.controller';
import { SystemDictController } from './dict/system-dict.controller';
import { SystemOptionController } from './option/system-option.controller';
import { SystemFileController } from './file/system-file.controller';
import { SystemStorageController } from './storage/system-storage.controller';
import { SystemClientController } from './client/system-client.controller';

/**
 * 系统管理模块，聚合用户、角色、菜单、部门等 /system/* 接口，
 * 行为与 Java/Go 后端保持一致，兼容同一前端。
 */
@Module({
  controllers: [
    SystemUserController,
    SystemRoleController,
    SystemMenuController,
    SystemDeptController,
    SystemDictController,
    SystemOptionController,
    SystemFileController,
    SystemStorageController,
    SystemClientController,
  ],
  providers: [PrismaService, TokenService, RSADecryptor, PasswordService],
})
export class SystemModule {}
