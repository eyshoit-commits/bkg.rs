import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { firstValueFrom } from 'rxjs';
import { ApiService } from '../../services/api.service';
import { AuthService } from '../../services/auth.service';
import { ApiKeyRecord } from '../../models/api.models';

@Component({
  selector: 'app-admin',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './admin.component.html',
  styleUrls: ['./admin.component.css']
})
export class AdminComponent implements OnInit {
  username = 'admin';
  password = '';
  loginError: string | null = null;
  keys: ApiKeyRecord[] = [];
  newKeyUser = 'admin';
  newKeyScopes = 'admin,llm.chat,llm.embed';
  createdKey: string | null = null;
  portTable: { service: string; port: string | number; status: string }[] = [];

  constructor(
    private readonly api: ApiService,
    private readonly auth: AuthService,
  ) {}

  ngOnInit(): void {
    if (this.auth.token) {
      void this.loadData();
    }
  }

  get authenticated(): boolean {
    return !!this.auth.token;
  }

  async login(): Promise<void> {
    this.loginError = null;
    try {
      const result = await firstValueFrom(this.api.login(this.username, this.password));
      this.auth.setToken(result.token);
      this.password = '';
      await this.loadData();
    } catch (error) {
      this.loginError = (error as Error).message;
    }
  }

  async loadData(): Promise<void> {
    this.keys = await firstValueFrom(this.api.listApiKeys());
    this.portTable = await firstValueFrom(this.api.portTable());
  }

  async createKey(): Promise<void> {
    try {
      const scopes = this.newKeyScopes.split(',').map((scope) => scope.trim()).filter(Boolean);
      const result = await firstValueFrom(this.api.createApiKey(this.newKeyUser, scopes));
      this.createdKey = result.token;
      await this.loadData();
    } catch (error) {
      this.loginError = (error as Error).message;
    }
  }

  async revokeKey(key: ApiKeyRecord): Promise<void> {
    await firstValueFrom(this.api.revokeApiKey(key.id));
    await this.loadData();
  }
}
