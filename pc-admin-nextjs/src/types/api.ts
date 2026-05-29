export interface ApiResponse<T> {
  code: string;
  data: T;
  msg: string;
  success: boolean;
  timestamp: string;
}

export interface PageResult<T> {
  list: T[];
  total: number;
}

export interface PageQuery {
  page?: number;
  size?: number;
  sort?: string[];
}
