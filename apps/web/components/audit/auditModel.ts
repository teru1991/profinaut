export type AuditResult = "SUCCESS" | "FAILED";

export type AuditEvent = {
  id: string;
  ts: string;
  actor: string;
  action: string;
  scope: string;
  reason: string;
  ttlMinutes: number;
  result: AuditResult;
  details?: string;
  trace_id?: string;
  run_id?: string;
  command_id?: string;
};

export type AuditFilters = {
  action?: string;
  result?: AuditResult;
  fromTs?: string;
  toTs?: string;
};
