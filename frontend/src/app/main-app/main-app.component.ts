import { BreakpointObserver, BreakpointState } from '@angular/cdk/layout';
import { Component, OnInit } from '@angular/core';
import { AuthService } from '../service/auth.service';
import { Router } from '@angular/router';

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
  ) {}

  large: boolean = false;
  loading = false;

  ngOnInit(): void {
    this.large = this.breakpointObserver.isMatched(BREAKPOINT);
    this.breakpointObserver
      .observe([BREAKPOINT])
      .subscribe((state: BreakpointState) => (this.large = state.matches));
  }

  async registerNew() {
    this.loading = true;
    let id = await this.authService.register_new();
    this.loading = false;
    this.router.navigate(['results', id]);
  }
}
