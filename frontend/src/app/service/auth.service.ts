import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';

const BASE_PATH = '';

@Injectable({
  providedIn: 'root',
})
export class AuthService {
  token?: string;

  constructor() {}

  async register_new(): Promise<string> {
    let res = await fetch(BASE_PATH + '/register', {
      method: 'POST',
    });
    if (res.status == 200) {
      let json = await res.json();
      if ('id' in json && 'auth' in json) {
        this.token = json['auth'];
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

  async validate_stored_for_id(id: string): Promise<boolean> {
    if (!this.has_token()) {
      return false;
    }
    return await this.validate_for_id(id, this.token || '');
  }

  async validate_for_id(id: string, auth: string): Promise<boolean> {
    let res = await fetch(BASE_PATH + '/validate/' + id, {
      method: 'HEAD',
      headers: { 'X-Auth': auth },
    });
    if (res.status == 200) {
      return true;
    }
    return false;
  }
}
