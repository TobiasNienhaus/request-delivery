import { BreakpointObserver, BreakpointState } from '@angular/cdk/layout';
import { Component, OnInit, ViewChild } from '@angular/core';
import { AuthService } from '../service/auth.service';
import { Router } from '@angular/router';
import { FormControl } from '@angular/forms';

const BREAKPOINT = '(min-width: 768px)';

@Component({
  selector: 'app-main-app',
  templateUrl: './main-app.component.html',
  styleUrls: ['./main-app.component.scss'],
})
export class MainAppComponent implements OnInit {
  constructor(
    private breakpointObserver: BreakpointObserver,
    private authService: AuthService,
    private router: Router
  ) {
    this.authService.clearToken();
  }

  newId = new FormControl('');
  newToken = new FormControl('');

  existingId = new FormControl('');
  existingToken = new FormControl('');

  large: boolean = false;
  loading = false;

  ngOnInit(): void {
    this.large = this.breakpointObserver.isMatched(BREAKPOINT);
    this.breakpointObserver
      .observe([BREAKPOINT])
      .subscribe((state: BreakpointState) => (this.large = state.matches));
  }

  async registerRandom() {
    this.loading = true;
    let id = await this.authService.registerRandom();
    this.loading = false;
    this.router.navigate(['results', id]);
  }

  async registerCustom() {
    this.loading = true;
    let id = await this.authService.registerNew(
      this.newId.value || '',
      this.newToken.value || ''
    );
    this.loading = false;
    this.router.navigate(['results', id]);
  }

  async connectExisting() {
    this.loading = true;
    const id = this.existingId.value || '';
    const token = this.existingToken.value || '';
    if (await this.authService.validate(id, token)) {
      this.authService.setToken(token);
      this.router.navigate(['results', id]);
    } else {
      this.loading = false;
    }
  }

  suspectedId(): string {
    return this.newId.value
      ? this.newId.value + 'X'.repeat(Math.max(0, 8 - this.newId.value.length))
      : 'X'.repeat(8);
  }

  idTooShort(): boolean {
    return (this.newId.value ? this.newId.value.length : 0) < 8;
  }
}
