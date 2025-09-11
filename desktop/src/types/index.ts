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

// Marketplace interfaces
export interface ProductRating {
  score: number;
  count: number;
}

export interface Product {
  id: string;
  name: string;
  description: string;
  category: string;
  developer: string;
  isPremium: boolean;
  rating: ProductRating;
  installs: number;
  actionText: string;
  icon?: string;
}

export type Category = 'All' | 'Popular' | 'New' | 'Premium' | 'Tools';

// Profile interfaces
export interface Activity {
  id: string;
  type: 'installation' | 'purchase' | 'usage' | 'contribution';
  title: string;
  description: string;
  timestamp: string;
}

export interface InstalledTool {
  id: string;
  name: string;
  type: 'performance' | 'security';
  installedDate: string;
}

export interface ProfileStats {
  toolsUsed: number;
  contributions: number;
  tasksCompleted: number;
  monthsActive: number;
  storageUsed: number;
  storageTotal: number;
  walletBalance: number;
  premiumCredits: number;
}