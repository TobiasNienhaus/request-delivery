import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { CookieService } from 'ngx-cookie-service';

const BASE_PATH = '';

@Injectable({
  providedIn: 'root',
})
export class AuthService {
  token?: string;

  constructor(private cookies: CookieService) {
    if (cookies.check('token')) {
      this.token = cookies.get('token');
    }
  }

  async registerNew(id: string, token: string): Promise<string> {
    this.clearToken();
    let res = await fetch(BASE_PATH + '/register', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ id: id, token: token }),
    });
    if (res.status < 300) {
      let json = await res.json();
      if ('id' in json && 'token' in json) {
        this.token = json['token'];
        this.cookies.set('token', json['token']);
        return json['id'];
      }
      throw 'Unauthorized';
    } else {
      throw 'Unauthorized';
    }
  }

  async registerRandom(): Promise<string> {
    this.cookies.delete('token');
    let res = await fetch(BASE_PATH + '/register/random', {
      method: 'POST',
    });
    if (res.status < 300) {
      let json = await res.json();
      if ('id' in json && 'token' in json) {
        this.token = json['token'];
        this.cookies.set('token', json['token']);
        return json['id'];
      }
      throw 'Unauthorized';
    } else {
      throw 'Unauthorized';
    }
  }

  has_token(): boolean {
    return this.token !== undefined;
  }

  async validateStored(id: string): Promise<boolean> {
    if (!this.has_token()) {
      return false;
    }
    return await this.validate(id, this.token || '');
  }

  async validate(id: string, auth: string): Promise<boolean> {
    let res = await fetch(BASE_PATH + '/validate/' + id, {
      method: 'HEAD',
      headers: { 'X-Auth': auth },
    });
    if (res.status == 200) {
      return true;
    }
    return false;
  }

  async clearToken() {
    this.cookies.delete('token');
    this.token = undefined;
  }

  async setToken(token: string) {
    this.cookies.set('token', token);
    this.token = token;
  }
}
