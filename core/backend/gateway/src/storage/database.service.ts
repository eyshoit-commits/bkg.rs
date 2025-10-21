import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import Database from 'better-sqlite3';
import { existsSync, mkdirSync } from 'fs';
import { dirname } from 'path';

@Injectable()
export class DatabaseService implements OnModuleInit {
  private readonly logger = new Logger(DatabaseService.name);
  private db!: InstanceType<typeof Database>;
  readonly path: string;

  constructor() {
    this.path = process.env.BKG_DATABASE_PATH ?? '/data/bkg.db';
  }

  onModuleInit() {
    const dir = dirname(this.path);
    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true });
    }
    this.db = new Database(this.path);
    this.db.pragma('journal_mode = WAL');
    this.createTables();
  }

  get connection(): InstanceType<typeof Database> {
    return this.db;
  }

  private createTables() {
    this.logger.log('Ensuring core tables exist');
    this.db
      .prepare(
        `CREATE TABLE IF NOT EXISTS plugins (
          name TEXT PRIMARY KEY,
          description TEXT,
          capabilities TEXT,
          autostart INTEGER DEFAULT 0,
          config TEXT
        )`,
      )
      .run();
    this.db
      .prepare(
        `CREATE TABLE IF NOT EXISTS api_keys (
          key TEXT PRIMARY KEY,
          user TEXT NOT NULL,
          scopes TEXT NOT NULL,
          created_at INTEGER NOT NULL
        )`,
      )
      .run();
    this.db
      .prepare(
        `CREATE TABLE IF NOT EXISTS users (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          password_hash TEXT NOT NULL
        )`,
      )
      .run();
  }
}
