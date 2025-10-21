import { Injectable } from '@angular/core';
import { BehaviorSubject } from 'rxjs';

const TOKEN_STORAGE_KEY = 'bkg_token';

@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly tokenSubject = new BehaviorSubject<string | null>(
    localStorage.getItem(TOKEN_STORAGE_KEY),
  );

  readonly token$ = this.tokenSubject.asObservable();

  get token(): string | null {
    return this.tokenSubject.value;
  }

  setToken(token: string | null): void {
    if (token) {
      localStorage.setItem(TOKEN_STORAGE_KEY, token);
    } else {
      localStorage.removeItem(TOKEN_STORAGE_KEY);
    }
    this.tokenSubject.next(token);
  }
}
