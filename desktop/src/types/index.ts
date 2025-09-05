export interface Task {
  id: string;
  name: string;
  status: 'running' | 'idle' | 'error';
  installed: string;
  runs: number;
  lastRun: string;
}

export type TaskStatus = 'all' | 'running' | 'idle' | 'error';

export interface LogEntry {
  id: string;
  message: string;
  timestamp: string;
  type: 'info' | 'success' | 'error';
}