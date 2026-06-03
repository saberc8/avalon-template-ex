import type { PageQuery } from "@/types/api";

export interface JobResp {
  id: number;
  name: string;
  groupName: string;
  taskType: 1 | 2;
  cronExpression: string;
  status: 1 | 2;
  concurrent: boolean;
  misfirePolicy: 1 | 2;
  maxRetry: number;
  timeoutSeconds: number;
  httpMethod: string;
  httpUrl: string;
  httpHeaders: Record<string, unknown>;
  httpBody: string;
  builtinKey: string;
  description: string;
  lastTriggerTime: string;
  nextTriggerTime: string;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
}

export interface JobCommand {
  name: string;
  groupName: string;
  taskType: number;
  cronExpression: string;
  status: number;
  concurrent: boolean;
  misfirePolicy: number;
  maxRetry: number;
  timeoutSeconds: number;
  httpMethod: string;
  httpUrl: string;
  httpHeaders: Record<string, unknown>;
  httpBody: string;
  builtinKey: string;
  description: string;
}

export interface JobQuery extends PageQuery {
  description?: string;
  groupName?: string;
  taskType?: number;
  status?: number;
  sort?: string[];
}

export interface JobTriggerResp {
  id: number;
  jobId: number;
  source: number;
  fireTime: string;
  status: number;
  attempt: number;
  maxAttempts: number;
  errorMsg: string;
  queuedTime: string;
  startTime: string;
  finishTime: string;
  createTime: string;
}

export interface JobLogResp {
  id: number;
  triggerId: number;
  jobId: number;
  attempt: number;
  status: number;
  executor: string;
  requestSnapshot: Record<string, unknown>;
  responseStatus: number;
  responseBody: string;
  errorMsg: string;
  startTime: string;
  finishTime: string;
  timeTaken: number;
}

export interface JobLogQuery extends PageQuery {
  status?: number;
}
