import { BreakpointObserver, BreakpointState } from '@angular/cdk/layout';
import { Component, OnInit } from '@angular/core';
import { AuthService } from '../service/auth.service';
import { Router } from '@angular/router';
import { FormControl, Validators } from '@angular/forms';
import { MatSnackBar } from '@angular/material/snack-bar';
import { LegalLinkComponent } from '../legal/legal-link/legal-link.component';

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
    private router: Router,
    private _snackbar: MatSnackBar
  ) {
    this.authService.clearToken();
  }

  newId = new FormControl('');
  newToken = new FormControl('');

  existingId = new FormControl('', [
    Validators.required,
    Validators.minLength(8),
  ]);
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
    try {
      let id = await this.authService.registerNew(
        this.newId.value || '',
        this.newToken.value || ''
      );
      this.router.navigate(['results', id]);
    } catch (e) {
      this._snackbar.open('Could not create credentials.', 'OK', {
        duration: 5000,
      });
    }
    this.loading = false;
  }

  async connectExisting() {
    this.loading = true;
    const id = this.existingId.value || '';
    const token = this.existingToken.value || '';
    if (await this.authService.validate(id, token)) {
      this.authService.setToken(token);
      this.router.navigate(['results', id]);
    } else {
      this._snackbar.open('Invalid Credentials.', 'OK', {
        duration: 5000,
      });
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
