// 公共类型。JSON 一律 snake_case，与后端 models.rs 一一对应，不做驼峰映射。

export type Mode = 'KZT' | 'SKZ' | 'VNL'

export const MODES: Mode[] = ['KZT', 'SKZ', 'VNL']

export interface Pagination {
  page: number
  per_page: number
  total: number
}

export interface PageResult<T> {
  data: T[]
  pagination: Pagination
}

export class ApiError extends Error {
  code: string
  status: number

  constructor(message: string, code: string, status: number) {
    super(message)
    this.name = 'ApiError'
    this.code = code
    this.status = status
  }
}
