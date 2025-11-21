export interface DeptResp {
  id: number;
  name: string;
  sort: number;
  status: number;
  isSystem: boolean;
  description: string;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
  parentId: number;
  children: DeptResp[];
}

export interface DeptTreeQuery {
  description?: string;
  status?: string;
}

export interface DeptReq {
  name: string;
  parentId: number;
  sort: number;
  status: number;
  description: string;
}

export interface DeleteDeptReq {
  ids: number[];
}

