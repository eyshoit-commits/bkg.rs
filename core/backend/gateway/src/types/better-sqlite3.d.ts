declare module 'better-sqlite3' {
  class Database {
    constructor(filename: string, options?: any);
    prepare(sql: string): Statement;
    exec(sql: string): this;
    pragma(pragma: string): any;
    close(): void;
  }

  interface Statement {
    run(...params: any[]): RunResult;
    get(...params: any[]): any;
    all(...params: any[]): any[];
  }

  interface RunResult {
    changes: number;
    lastInsertRowid: number;
  }

  export = Database;
}
