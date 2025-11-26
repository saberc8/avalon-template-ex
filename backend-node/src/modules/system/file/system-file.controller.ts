import {
  Body,
  Controller,
  Delete,
  Get,
  Headers,
  Param,
  Post,
  Put,
  Query,
  UploadedFile,
  UseInterceptors,
} from '@nestjs/common';
import { FileInterceptor } from '@nestjs/platform-express';
import { PrismaService } from '../../../shared/prisma/prisma.service';
import { ok, fail } from '../../../shared/api-response/api-response';
import { IdsRequest, PageResult } from '../user/dto';
import { nextId } from '../../../shared/id/id';
import {
  FileItem,
  FileListQuery,
  FileStatisticsResp,
  FileUploadResp,
  FileDirCalcSizeResp,
} from './dto';
import { TokenService } from '../../auth/jwt/jwt.service';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as crypto from 'node:crypto';

/**
 * 文件管理接口集合，实现 /system/file 列表查询功能，
 * 行为参考 backend-go/internal/interfaces/http/file_handler.go#ListFile。
 */
@Controller()
export class SystemFileController {
  constructor(
    private readonly prisma: PrismaService,
    private readonly tokenService: TokenService,
  ) {}

  /** 从 Authorization 头解析当前登录用户 ID，失败时返回 0。 */
  private currentUserId(authorization?: string) {
    const claims = this.tokenService.parse(authorization || undefined);
    if (!claims) return 0;
    return claims.userId;
  }

  /** POST /system/file/upload 上传文件 */
  @Post('/system/file/upload')
  @UseInterceptors(FileInterceptor('file'))
  async uploadFile(
    @Headers('authorization') authorization: string | undefined,
    @UploadedFile() file?: any,
    @Body('parentPath') parentPathBody?: string,
  ) {
    const userId = this.currentUserId(authorization);
    if (!userId) {
      return fail('401', '未授权，请重新登录');
    }
    if (!file) {
      return fail('400', '文件不能为空');
    }

    const parentPath = this.normalizeParentPath(parentPathBody || '/');
    const ext = this.extensionFromFilename(file.originalname);
    const newId = nextId();
    const storedName = ext ? `${newId.toString()}.${ext}` : newId.toString();

    // 查询默认存储配置，缺失时退回本地 ./data/file。
    const storageCfg = await this.getDefaultStorage().catch(() => null);
    if (!storageCfg) {
      return fail('500', '获取存储配置失败');
    }

    let fullPath = '';
    let sha = '';
    let size = 0;
    let contentType = file.mimetype || '';

    try {
      if (storageCfg.type === storageTypeOSS) {
        // 当前 Node 版暂不直接对接对象存储，实现与 Go 版语义一致的本地回退。
        const saved = await this.saveToLocal(
          storageCfg.bucketName,
          parentPath,
          storedName,
          file,
        );
        fullPath = saved.fullPath;
        sha = saved.sha;
        size = saved.size;
        if (!contentType) {
          contentType = saved.contentType;
        }
      } else {
        const saved = await this.saveToLocal(
          storageCfg.bucketName,
          parentPath,
          storedName,
          file,
        );
        fullPath = saved.fullPath;
        sha = saved.sha;
        size = saved.size;
        if (!contentType) {
          contentType = saved.contentType;
        }
      }
    } catch {
      return fail('500', '保存文件失败');
    }

    const now = new Date();
    const fileType = this.detectFileType(ext, contentType);
    const insertSql = `
INSERT INTO sys_file (
    id, name, original_name, size, parent_path, path, extension, content_type,
    type, sha256, metadata, thumbnail_name, thumbnail_size, thumbnail_metadata,
    storage_id, create_user, create_time
) VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8,
    $9, $10, $11, $12, $13, $14,
    $15, $16, $17
);`;

    const fileId = nextId();
    const meta = '';
    try {
      await this.prisma.$executeRawUnsafe(
        insertSql,
        fileId,
        storedName,
        file.originalname,
        BigInt(size),
        parentPath,
        fullPath,
        ext,
        contentType,
        fileType,
        sha,
        meta,
        '',
        null,
        '',
        storageCfg.id,
        BigInt(userId),
        now,
      );
    } catch {
      return fail('500', '保存文件记录失败');
    }

    const url = this.buildStorageFileURL(storageCfg, fullPath);
    const resp: FileUploadResp = {
      id: fileId.toString(),
      url,
      thUrl: url,
      metadata: {},
    };
    return ok(resp);
  }

  /** GET /system/file 文件分页列表 */
  @Get('/system/file')
  async listFile(@Query() query: FileListQuery) {
    try {
      const originalName = (query.originalName || '').trim();
      const typeStr = (query.type || '').trim();
      const parentPathRaw = (query.parentPath || '').trim();

      let page = Number(query.page ?? '1');
      let size = Number(query.size ?? '30');
      if (!Number.isFinite(page) || page <= 0) page = 1;
      if (!Number.isFinite(size) || size <= 0) size = 30;

      let where = 'WHERE 1=1';
      const params: any[] = [];
      let idx = 1;

      if (originalName) {
        where += ` AND f.original_name ILIKE $${idx}`;
        params.push(`%${originalName}%`);
        idx++;
      }

      if (typeStr && typeStr !== '0') {
        const t = Number(typeStr);
        if (Number.isFinite(t) && t > 0) {
          where += ` AND f.type = $${idx}`;
          params.push(t);
          idx++;
        }
      }

      if (parentPathRaw) {
        where += ` AND f.parent_path = $${idx}`;
        params.push(this.normalizeParentPath(parentPathRaw));
        idx++;
      }

      const countSql = `SELECT COUNT(*)::bigint AS total FROM sys_file AS f ${where};`;
      const countRows = await this.prisma.$queryRawUnsafe<
        { total: bigint }[]
      >(countSql, ...params);
      const total = countRows.length ? Number(countRows[0].total) : 0;
      if (!total) {
        const empty: PageResult<FileItem> = { list: [], total: 0 };
        return ok(empty);
      }

      const offset = (page - 1) * size;
      const limitPos = idx;
      const offsetPos = idx + 1;
      const listParams = [...params, size, offset];

      const sql = `
SELECT f.id,
       f.name,
       f.original_name,
       f.size,
       f.parent_path,
       f.path,
       COALESCE(f.extension, '')           AS extension,
       COALESCE(f.content_type, '')        AS content_type,
       f.type,
       COALESCE(f.sha256, '')              AS sha256,
       COALESCE(f.metadata, '')            AS metadata,
       COALESCE(f.thumbnail_name, '')      AS thumbnail_name,
       f.thumbnail_size,
       COALESCE(f.thumbnail_metadata, '')  AS thumbnail_metadata,
       f.storage_id,
       f.create_time,
       COALESCE(cu.nickname, '')          AS create_user_string,
       f.update_time,
       COALESCE(uu.nickname, '')          AS update_user_string
FROM sys_file AS f
LEFT JOIN sys_user AS cu ON cu.id = f.create_user
LEFT JOIN sys_user AS uu ON uu.id = f.update_user
${where}
ORDER BY f.type ASC, f.update_time DESC NULLS LAST, f.id DESC
LIMIT $${limitPos} OFFSET $${offsetPos};
`;

      const rows = await this.prisma.$queryRawUnsafe<
        {
          id: bigint;
          name: string;
          original_name: string;
          size: bigint | null;
          parent_path: string;
          path: string;
          extension: string;
          content_type: string;
          type: number;
          sha256: string;
          metadata: string;
          thumbnail_name: string;
          thumbnail_size: bigint | null;
          thumbnail_metadata: string;
          storage_id: bigint | null;
          create_time: Date;
          create_user_string: string;
          update_time: Date | null;
          update_user_string: string;
        }[]
      >(sql, ...listParams);

      const list: FileItem[] = [];
      for (const r of rows) {
        const item: FileItem = {
          id: Number(r.id),
          name: r.name,
          originalName: r.original_name,
          size: r.size ? Number(r.size) : 0,
          url: '',
          parentPath: r.parent_path,
          path: r.path,
          sha256: r.sha256,
          contentType: r.content_type,
          metadata: r.metadata,
          thumbnailSize: r.thumbnail_size ? Number(r.thumbnail_size) : 0,
          thumbnailName: r.thumbnail_name,
          thumbnailMetadata: r.thumbnail_metadata,
          thumbnailUrl: '',
          extension: r.extension,
          type: Number(r.type),
          storageId: r.storage_id ? Number(r.storage_id) : 0,
          storageName: '',
          createUserString: r.create_user_string,
          createTime: r.create_time.toISOString(),
          updateUserString: r.update_user_string,
          updateTime: r.update_time ? r.update_time.toISOString() : '',
        };

        let storageCfg: StorageConfig | null = null;
        if (item.storageId > 0) {
          storageCfg = await this.getStorageById(item.storageId).catch(
            () => null,
          );
        }

        if (storageCfg) {
          item.storageName = storageCfg.name;
        } else {
          item.storageName = '本地存储';
        }

        item.url = this.buildStorageFileURL(storageCfg, item.path);
        if (item.thumbnailName) {
          let parent = item.parentPath || '';
          if (parent === '/') {
            parent = '';
          }
          const thumbPath = `${parent}/${item.thumbnailName}`;
          item.thumbnailUrl = this.buildStorageFileURL(storageCfg, thumbPath);
        } else {
          item.thumbnailUrl = item.url;
        }

        list.push(item);
      }

      const resp: PageResult<FileItem> = {
        list,
        total,
      };
      return ok(resp);
    } catch {
      return fail('500', '查询文件失败');
    }
  }

  /** PUT /system/file/:id 重命名文件（仅修改 original_name）。 */
  @Put('/system/file/:id')
  async updateFile(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: { originalName?: string },
  ) {
    const userId = this.currentUserId(authorization);
    if (!userId) {
      return fail('401', '未授权，请重新登录');
    }
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const originalName = (body?.originalName ?? '').trim();
    if (!originalName) {
      return fail('400', '名称不能为空');
    }

    try {
      await this.prisma.$executeRawUnsafe(
        `
UPDATE sys_file
   SET original_name = $1,
       update_user   = $2,
       update_time   = $3
 WHERE id            = $4;
`,
        originalName,
        BigInt(userId),
        new Date(),
        BigInt(id),
      );
    } catch {
      return fail('500', '重命名失败');
    }

    return ok(true);
  }

  /** DELETE /system/file 删除文件/文件夹（批量），行为参考 Go 版 DeleteFile。 */
  @Delete('/system/file')
  async deleteFile(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: IdsRequest,
  ) {
    const userId = this.currentUserId(authorization);
    if (!userId) {
      return fail('401', '未授权，请重新登录');
    }
    if (!body?.ids?.length) {
      return fail('400', 'ID 列表不能为空');
    }
    const ids = body.ids.filter((v) => Number.isFinite(v) && v > 0);
    if (!ids.length) {
      return fail('400', 'ID 列表不能为空');
    }

    type FileRow = {
      id: bigint;
      name: string;
      path: string | null;
      parent_path: string;
      type: number;
      storage_id: bigint | null;
    };

    try {
      const rows = await this.prisma.$queryRaw<
        FileRow[]
      >`SELECT id, name, path, parent_path, type, storage_id FROM sys_file WHERE id = ANY(${ids as any}::bigint[]);`;

      const toDeleteFiles: FileRow[] = [];

      for (const row of rows) {
        const fileType = Number(row.type);
        if (fileType === 0) {
          // 目录：校验是否为空
          const childRows = await this.prisma.$queryRaw<
            { exists: number }[]
          >`SELECT 1::int AS exists FROM sys_file WHERE parent_path = ${row.path} LIMIT 1;`;
          if (childRows.length) {
            return fail(
              '400',
              `文件夹 [${row.name}] 不为空，请先删除文件夹下的内容`,
            );
          }
        } else {
          toDeleteFiles.push(row);
        }
      }

      // 删除数据库记录
      await this.prisma.$executeRawUnsafe(
        `DELETE FROM sys_file WHERE id = ANY($1::bigint[]);`,
        ids.map((v) => BigInt(v)),
      );

      // 尽力删除物理文件（仅本地存储场景）
      for (const row of toDeleteFiles) {
        if (!row.path) continue;
        const storageId = row.storage_id ? Number(row.storage_id) : 0;
        const storageCfg = storageId
          ? await this.getStorageById(storageId).catch(() => null)
          : await this.getDefaultStorage().catch(() => null);
        if (!storageCfg) continue;

        const relative = row.path.replace(/^\/+/u, '');
        const bucket = (storageCfg.bucketName || './data/file').trim() || './data/file';
        const abs = path.join(bucket, relative);
        try {
          await fs.promises.unlink(abs);
        } catch {
          // 忽略物理删除失败，保持逻辑删除成功
        }
      }
    } catch {
      return fail('500', '删除文件失败');
    }

    return ok(true);
  }

  /** POST /system/file/dir 创建文件夹。 */
  @Post('/system/file/dir')
  async createDir(
    @Headers('authorization') authorization: string | undefined,
    @Body()
    body: {
      parentPath?: string;
      originalName?: string;
    },
  ) {
    const userId = this.currentUserId(authorization);
    if (!userId) {
      return fail('401', '未授权，请重新登录');
    }
    const originalName = (body?.originalName ?? '').trim();
    if (!originalName) {
      return fail('400', '名称不能为空');
    }
    const parentRaw = body?.parentPath ?? '/';
    const parentPath = this.normalizeParentPath(parentRaw);

    // 校验同一父级下文件夹重名
    const existsRows = await this.prisma.$queryRaw<
      { exists: number }[]
    >`SELECT 1::int AS exists FROM sys_file WHERE parent_path = ${parentPath} AND name = ${originalName} AND type = 0 LIMIT 1;`;
    if (existsRows.length) {
      return fail('400', '文件夹已存在');
    }

    const now = new Date();
    const dirId = nextId();
    const pathVal =
      parentPath === '/' ? `/${originalName}` : `${parentPath}/${originalName}`;

    try {
      await this.prisma.$executeRawUnsafe(
        `
INSERT INTO sys_file (
    id, name, original_name, size, parent_path, path, extension, content_type,
    type, sha256, metadata, thumbnail_name, thumbnail_size, thumbnail_metadata,
    storage_id, create_user, create_time
) VALUES (
    $1, $2, $3, NULL, $4, $5, NULL, NULL,
    0, '', '', '', NULL, '',
    1, $6, $7
);
`,
        dirId,
        originalName,
        originalName,
        parentPath,
        pathVal,
        BigInt(userId),
        now,
      );
    } catch {
      return fail('500', '创建文件夹失败');
    }

    return ok(true);
  }

  /** GET /system/file/check?fileHash=... ，根据 sha256 检查文件是否已存在。 */
  @Get('/system/file/check')
  async checkFile(@Query('fileHash') fileHash?: string) {
    const hash = (fileHash || '').trim();
    if (!hash) {
      return ok<any | null>(null as any);
    }

    try {
      const rows = await this.prisma.$queryRaw<
        {
          id: bigint;
          name: string;
          original_name: string;
          size: bigint | null;
          parent_path: string;
          path: string;
          extension: string;
          content_type: string;
          type: number;
          sha256: string;
          metadata: string;
          thumbnail_name: string;
          thumbnail_size: bigint | null;
          thumbnail_metadata: string;
          storage_id: bigint | null;
          create_time: Date;
          create_user_string: string;
          update_time: Date | null;
          update_user_string: string;
        }[]
      >`
SELECT f.id,
       f.name,
       f.original_name,
       f.size,
       f.parent_path,
       f.path,
       COALESCE(f.extension, '')           AS extension,
       COALESCE(f.content_type, '')        AS content_type,
       f.type,
       COALESCE(f.sha256, '')              AS sha256,
       COALESCE(f.metadata, '')            AS metadata,
       COALESCE(f.thumbnail_name, '')      AS thumbnail_name,
       f.thumbnail_size,
       COALESCE(f.thumbnail_metadata, '')  AS thumbnail_metadata,
       f.storage_id,
       f.create_time,
       COALESCE(cu.nickname, '')          AS create_user_string,
       f.update_time,
       COALESCE(uu.nickname, '')          AS update_user_string
FROM sys_file AS f
LEFT JOIN sys_user AS cu ON cu.id = f.create_user
LEFT JOIN sys_user AS uu ON uu.id = f.update_user
WHERE f.sha256 = ${hash}
LIMIT 1;
`;
      if (!rows.length) {
        return ok<any | null>(null as any);
      }
      const r = rows[0];
      const item: FileItem = {
        id: Number(r.id),
        name: r.name,
        originalName: r.original_name,
        size: r.size ? Number(r.size) : 0,
        url: '',
        parentPath: r.parent_path,
        path: r.path,
        sha256: r.sha256,
        contentType: r.content_type,
        metadata: r.metadata,
        thumbnailSize: r.thumbnail_size ? Number(r.thumbnail_size) : 0,
        thumbnailName: r.thumbnail_name,
        thumbnailMetadata: r.thumbnail_metadata,
        thumbnailUrl: '',
        extension: r.extension,
        type: Number(r.type),
        storageId: r.storage_id ? Number(r.storage_id) : 0,
        storageName: '',
        createUserString: r.create_user_string,
        createTime: r.create_time.toISOString(),
        updateUserString: r.update_user_string,
        updateTime: r.update_time ? r.update_time.toISOString() : '',
      };

      let storageCfg: StorageConfig | null = null;
      if (item.storageId > 0) {
        storageCfg = await this.getStorageById(item.storageId).catch(
          () => null,
        );
      }
      if (storageCfg) {
        item.storageName = storageCfg.name;
      } else {
        item.storageName = '本地存储';
      }
      item.url = this.buildStorageFileURL(storageCfg, item.path);
      if (item.thumbnailName) {
        let parent = item.parentPath || '';
        if (parent === '/') {
          parent = '';
        }
        const thumbPath = `${parent}/${item.thumbnailName}`;
        item.thumbnailUrl = this.buildStorageFileURL(storageCfg, thumbPath);
      } else {
        item.thumbnailUrl = item.url;
      }

      return ok<FileItem | null>(item);
    } catch {
      return fail('500', '查询文件失败');
    }
  }

  /** GET /system/file/statistics 文件资源统计信息 */
  @Get('/system/file/statistics')
  async statistics() {
    try {
      const rows = await this.prisma.$queryRaw<
        { type: number; number: bigint; size: bigint }[]
      >`
SELECT type,
       COUNT(1)                AS number,
       COALESCE(SUM(size), 0)  AS size
FROM sys_file
WHERE type <> 0
GROUP BY type;
`;

      if (!rows.length) {
        const empty: FileStatisticsResp = {
          type: '',
          size: 0,
          number: 0,
          unit: '',
          data: [],
        };
        return ok(empty);
      }

      let totalSize = 0;
      let totalNumber = 0;
      const children: FileStatisticsResp[] = [];

      for (const r of rows) {
        const size = Number(r.size ?? 0);
        const number = Number(r.number ?? 0);
        totalSize += size;
        totalNumber += number;
        children.push({
          type: String(r.type),
          size,
          number,
          unit: '',
          data: [],
        });
      }

      const resp: FileStatisticsResp = {
        type: '',
        size: totalSize,
        number: totalNumber,
        unit: '',
        data: children,
      };
      return ok(resp);
    } catch {
      return fail('500', '查询文件统计失败');
    }
  }

  /** GET /system/file/dir/:id/size 计算文件夹大小。 */
  @Get('/system/file/dir/:id/size')
  async calcDirSize(@Param('id') idParam: string) {
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const dirRows = await this.prisma.$queryRaw<
      { path: string | null; type: number }[]
    >`SELECT path, type FROM sys_file WHERE id = ${BigInt(id)};`;
    if (!dirRows.length) {
      return fail('404', '文件夹不存在');
    }
    const dir = dirRows[0];
    if (Number(dir.type) !== 0) {
      return fail('400', 'ID 不是文件夹，无法计算大小');
    }
    const pathVal = (dir.path || '').trim();
    if (!pathVal) {
      const resp: FileDirCalcSizeResp = { size: 0 };
      return ok(resp);
    }

    const prefix = `${pathVal.replace(/\/+$/u, '')}/%`;
    const sizeRows = await this.prisma.$queryRaw<
      { total: bigint | null }[]
    >`SELECT COALESCE(SUM(size), 0)::bigint AS total FROM sys_file WHERE type <> 0 AND path LIKE ${prefix};`;
    const total = sizeRows.length && sizeRows[0].total ? Number(sizeRows[0].total) : 0;
    const resp: FileDirCalcSizeResp = { size: total };
    return ok(resp);
  }

  /** 归一化父级路径，形如 / 或 /a/b，不保留末尾斜杠。 */
  private normalizeParentPath(p: string): string {
    let val = (p || '').trim();
    if (!val) return '/';
    if (!val.startsWith('/')) {
      val = `/${val}`;
    }
    if (val.length > 1) {
      val = val.replace(/\/+$/u, '');
    }
    return val;
  }

  /** 本地文件基础 URL 前缀，默认为 /file，可通过 FILE_BASE_URL 环境变量覆盖。 */
  private fileBaseURLPrefix(): string {
    let prefix = process.env.FILE_BASE_URL || '/file';
    prefix = prefix.trim();
    if (!prefix.startsWith('/')) {
      prefix = `/${prefix}`;
    }
    return prefix.replace(/\/+$/u, '');
  }

  /** 构建本地存储访问 URL。 */
  private buildLocalFileURL(path: string): string {
    if (!path) return '';
    let p = path;
    if (!p.startsWith('/')) {
      p = `/${p}`;
    }
    return `${this.fileBaseURLPrefix()}${p}`;
  }

  /** 根据存储配置构建文件访问 URL，逻辑与 Go 版 buildStorageFileURL 保持一致。 */
  private buildStorageFileURL(
    storage: StorageConfig | null,
    fullPath: string,
  ): string {
    if (!storage) {
      return this.buildLocalFileURL(fullPath);
    }
    // 2 = 对象存储，其余视为本地存储
    if (storage.type === storageTypeOSS) {
      let domain = (storage.domain || '').trim();
      if (!domain) {
        return this.buildLocalFileURL(fullPath);
      }
      domain = domain.replace(/\/+$/u, '');
      const key = (fullPath || '').replace(/^\/+/u, '');
      return `${domain}/${key}`;
    }
    return this.buildLocalFileURL(fullPath);
  }

  /** 根据存储 ID 查询 sys_storage 配置，缺失或异常时返回 null。 */
  private async getStorageById(
    id: number,
  ): Promise<StorageConfig | null> {
    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        name: string;
        code: string;
        type: number;
        access_key: string;
        secret_key: string;
        endpoint: string;
        bucket_name: string;
        domain: string;
        region: string;
        is_default: boolean;
        status: number;
      }[]
    >`
SELECT id,
       name,
       code,
       type,
       COALESCE(access_key, '')   AS access_key,
       COALESCE(secret_key, '')   AS secret_key,
       COALESCE(endpoint, '')     AS endpoint,
       COALESCE(bucket_name, '')  AS bucket_name,
       COALESCE(domain, '')       AS domain,
       COALESCE(region, '')       AS region,
       COALESCE(is_default, FALSE) AS is_default,
       COALESCE(status, 1)        AS status
FROM sys_storage
WHERE id = ${BigInt(id)};
`;
    if (!rows.length) return null;
    const r = rows[0];
    return {
      id: Number(r.id),
      name: r.name,
      code: r.code,
      type: Number(r.type),
      accessKey: r.access_key,
      secretKey: r.secret_key,
      endpoint: r.endpoint,
      bucketName: r.bucket_name || './data/file',
      domain: r.domain,
      region: r.region,
      isDefault: !!r.is_default,
      status: Number(r.status),
    };
  }

  /** 查询默认存储配置，若不存在则回退到本地存储 ./data/file。 */
  private async getDefaultStorage(): Promise<StorageConfig> {
    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        name: string;
        code: string;
        type: number;
        access_key: string;
        secret_key: string;
        endpoint: string;
        bucket_name: string;
        domain: string;
        region: string;
        is_default: boolean;
        status: number;
      }[]
    >`
SELECT id,
       name,
       code,
       type,
       COALESCE(access_key, '')   AS access_key,
       COALESCE(secret_key, '')   AS secret_key,
       COALESCE(endpoint, '')     AS endpoint,
       COALESCE(bucket_name, '')  AS bucket_name,
       COALESCE(domain, '')       AS domain,
       COALESCE(region, '')       AS region,
       COALESCE(is_default, FALSE) AS is_default,
       COALESCE(status, 1)        AS status
FROM sys_storage
WHERE is_default = TRUE
LIMIT 1;
`;
    if (!rows.length) {
      return {
        id: 1,
        name: '本地存储',
        code: 'local',
        type: storageTypeLocal,
        accessKey: '',
        secretKey: '',
        endpoint: '',
        bucketName: './data/file',
        domain: '',
        region: '',
        isDefault: true,
        status: 1,
      };
    }
    const r = rows[0];
    return {
      id: Number(r.id),
      name: r.name,
      code: r.code,
      type: Number(r.type),
      accessKey: r.access_key,
      secretKey: r.secret_key,
      endpoint: r.endpoint,
      bucketName: r.bucket_name || './data/file',
      domain: r.domain,
      region: r.region,
      isDefault: !!r.is_default,
      status: Number(r.status),
    };
  }

  /** 从文件名中提取扩展名（不含点）。 */
  private extensionFromFilename(name: string): string {
    const ext = path.extname(name || '').toLowerCase();
    return ext.startsWith('.') ? ext.slice(1) : ext;
  }

  /** 识别文件类型，行为参考 Go 版 detectFileType。 */
  private detectFileType(ext: string, contentType: string): number {
    const lowerExt = (ext || '').toLowerCase();
    const ct = (contentType || '').toLowerCase();
    if (ct.startsWith('image/')) return 2;
    if (ct.startsWith('video/')) return 4;
    if (ct.startsWith('audio/')) return 5;
    if (['jpg', 'jpeg', 'png', 'gif'].includes(lowerExt)) return 2;
    if (
      [
        'doc',
        'docx',
        'xls',
        'xlsx',
        'ppt',
        'pptx',
        'pdf',
        'txt',
      ].includes(lowerExt)
    ) {
      return 3;
    }
    return 1;
  }

  /**
   * 将上传文件保存到本地存储，返回逻辑路径（存入 DB）、大小、SHA256 与内容类型。
   * 行为与 Go 版 saveToLocal 保持一致。
   */
  private async saveToLocal(
    bucketPath: string,
    parentPath: string,
    storedName: string,
    file: any,
  ): Promise<{ fullPath: string; sha: string; size: number; contentType: string }> {
    const normalizedParent = this.normalizeParentPath(parentPath);
    const fullPath =
      normalizedParent === '/'
        ? `/${storedName}`
        : `${normalizedParent}/${storedName}`;

    const base = (bucketPath || './data/file').trim() || './data/file';
    const relative = fullPath.replace(/^\/+/u, '');
    const dstPath = path.join(base, relative);
    await fs.promises.mkdir(path.dirname(dstPath), { recursive: true });

    const hash = crypto.createHash('sha256');
    let size = 0;
    // 使用 buffer 写入，兼容 Multer 默认内存存储。
    if (file.buffer) {
      await fs.promises.writeFile(dstPath, file.buffer);
      hash.update(file.buffer);
      size = file.buffer.length;
    } else {
      // 非内存模式时 fallback 使用流复制。
      const src = fs.createReadStream(file.path);
      const dst = fs.createWriteStream(dstPath);
      await new Promise<void>((resolve, reject) => {
        src
          .on('data', (chunk) => {
            hash.update(chunk);
            size += chunk.length;
          })
          .on('error', reject)
          .pipe(dst)
          .on('error', reject)
          .on('finish', resolve);
      });
    }

    const sha = hash.digest('hex');
    const contentType = file.mimetype || '';
    return { fullPath, sha, size, contentType };
  }
}

/** 存储配置结构，映射 sys_storage 表。 */
interface StorageConfig {
  id: number;
  name: string;
  code: string;
  type: number;
  accessKey: string;
  secretKey: string;
  endpoint: string;
  bucketName: string;
  domain: string;
  region: string;
  isDefault: boolean;
  status: number;
}

// 存储类型常量：1=本地，2=对象存储（MinIO/S3 等）
const storageTypeLocal = 1;
const storageTypeOSS = 2;
