import type * as T from './type'
import http from '@/utils/http'

export type * from './type'

const BASE_URL = '/system/banner'

/** @desc 查询 Banner 列表（分页） */
export function listBanner(query: T.BannerPageQuery) {
  return http.get<PageRes<T.BannerResp[]>>(BASE_URL, query)
}

/** @desc 查询 Banner 详情 */
export function getBanner(id: string) {
  return http.get<T.BannerResp>(`${BASE_URL}/${id}`)
}

/** @desc 新增 Banner */
export function addBanner(data: any) {
  return http.post(BASE_URL, data)
}

/** @desc 修改 Banner */
export function updateBanner(data: any, id: string) {
  return http.put(`${BASE_URL}/${id}`, data)
}

/** @desc 删除 Banner */
export function deleteBanner(id: string) {
  return http.del(`${BASE_URL}`, { ids: [id] })
}

