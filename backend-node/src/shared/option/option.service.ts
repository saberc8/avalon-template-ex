import { Injectable } from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';

/**
 * 系统参数配置服务，基于 sys_option 表。
 *
 * 说明：
 * - 与 Java 版 OptionService 行为保持一致：优先使用 value，若为空则回退 default_value。
 * - 仅负责简单的读取逻辑，不做业务判断，由上层自行处理。
 */
@Injectable()
export class OptionService {
  constructor(private readonly prisma: PrismaService) {}

  /**
   * 根据 code 获取字符串配置值，若不存在则返回空字符串。
   */
  async getValueByCode(code: string): Promise<string> {
    const c = (code || '').trim();
    if (!c) {
      return '';
    }
    const row = await this.prisma.sys_option.findFirst({
      where: { code: c },
      select: { value: true, default_value: true },
    });
    if (!row) {
      return '';
    }
    return (row.value ?? row.default_value ?? '').toString();
  }

  /**
   * 根据 code 获取整型配置值，解析失败时返回 0。
   */
  async getIntValueByCode(code: string): Promise<number> {
    const str = await this.getValueByCode(code);
    if (!str) {
      return 0;
    }
    const n = Number.parseInt(str, 10);
    if (!Number.isFinite(n)) {
      return 0;
    }
    return n;
  }

  /**
   * 根据类别获取同一类别下的所有配置，返回 code -> value 映射。
   */
  async getByCategory(category: string): Promise<Record<string, string>> {
    const cat = (category || '').trim();
    if (!cat) {
      return {};
    }
    const rows = await this.prisma.sys_option.findMany({
      where: { category: cat },
      select: { code: true, value: true, default_value: true },
      orderBy: { id: 'asc' },
    });
    const map: Record<string, string> = {};
    for (const r of rows) {
      const v = (r.value ?? r.default_value ?? '').toString();
      map[r.code] = v;
    }
    return map;
  }
}

