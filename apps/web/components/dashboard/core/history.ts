import type { Workspace } from "./model";

export class WorkspaceHistory {
  private past: Workspace[] = [];
  private future: Workspace[] = [];

  constructor(private readonly maxSteps = 100) {}

  push(state: Workspace): void {
    this.past.push(structuredClone(state));
    if (this.past.length > this.maxSteps) {
      this.past.shift();
    }
    this.future = [];
  }

  undo(current: Workspace): Workspace | null {
    const previous = this.past.pop();
    if (!previous) {
      return null;
    }
    this.future.push(structuredClone(current));
    return structuredClone(previous);
  }

  redo(current: Workspace): Workspace | null {
    const next = this.future.pop();
    if (!next) {
      return null;
    }
    this.past.push(structuredClone(current));
    return structuredClone(next);
  }

  canUndo(): boolean {
    return this.past.length > 0;
  }

  canRedo(): boolean {
    return this.future.length > 0;
  }

  clear(): void {
    this.past = [];
    this.future = [];
  }
}
