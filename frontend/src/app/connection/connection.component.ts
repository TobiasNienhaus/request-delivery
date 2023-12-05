import { Component, OnInit, ViewChild } from '@angular/core';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import { RequestData, DEFAULT_REQUEST } from '../model/request';
import { MatSnackBar } from '@angular/material/snack-bar';
import { IClipboardResponse } from 'ngx-clipboard';
import { AuthService } from '../service/auth.service';
import {
  MatBottomSheet,
  MatBottomSheetModule,
  MatBottomSheetRef,
} from '@angular/material/bottom-sheet';
import { ConnectionInfoBottomSheetComponent } from './connection-info-bottom-sheet/connection-info-bottom-sheet.component';
import { firstValueFrom } from 'rxjs';
import { BreakpointObserver } from '@angular/cdk/layout';
import { MatSidenav } from '@angular/material/sidenav';

const BREAKPOINT = '(min-width: 768px)';

@Component({
  selector: 'app-connection',
  templateUrl: './connection.component.html',
  styleUrls: ['./connection.component.scss'],
})
export class ConnectionComponent implements OnInit {
  selectedEvent?: RequestData;
  websocket?: WebSocket;
  id?: string;
  broken = true;
  isMobile = true;

  @ViewChild(MatSidenav)
  sidenav!: MatSidenav;

  events: RequestData[] = [];

  constructor(
    private route: ActivatedRoute,
    private _snackbar: MatSnackBar,
    private _bottomSheet: MatBottomSheet,
    private authService: AuthService,
    private router: Router,
    private observer: BreakpointObserver
  ) {
    this.isMobile = !observer.isMatched(BREAKPOINT);
    if (!authService.has_token()) {
      router.navigate(['ui']);
      return;
    }
    this.selectedEvent = this.events[0];
    firstValueFrom(this.route.paramMap).then((params) => {
      let id = params.get('id');
      this.id = id || undefined;
      if (this.id) {
        this.createNewWebsocketConnection();
        this.broken = false;
      } else {
        console.error('Missing ID');
      }
    });
  }

  ngOnInit() {
    this.observer.observe(BREAKPOINT).subscribe((screenSize) => {
      this.isMobile = !screenSize.matches;
    });
  }

  toggleMenu() {
    this.sidenav.toggle();
  }

  get eventsString(): string {
    return JSON.stringify(this.events);
  }

  cbUrl(id: string): string {
    return `${location.origin}/send/${id}`;
  }

  onCopy($event: IClipboardResponse) {
    this._snackbar.open(
      $event.isSuccess ? `Copied: ${$event.content}` : 'Failed to Copy',
      'Ok',
      {
        duration: 5000,
      }
    );
  }

  openInfoSheet() {
    this._bottomSheet.open(ConnectionInfoBottomSheetComponent, {
      data: { id: this.id },
    });
  }

  closeConnection() {
    if (this.websocket) {
      this.websocket.close(1000, 'CLIENT');
    }
  }

  tryReopenConnection() {
    if (this.broken) return;
    this.createNewWebsocketConnection();
    this._snackbar.open('Reopened WebSocket Connection', 'Ok', {
      duration: 5000,
    });
  }

  hasConnection() {
    return (
      this.websocket &&
      this.websocket.readyState in [WebSocket.OPEN, WebSocket.CONNECTING]
    );
  }

  goHome() {
    this.authService.clearToken();
    this.router.navigate(['/']);
    this.broken = true;
  }

  createNewWebsocketConnection() {
    this.websocket = new WebSocket(
      `ws://${location.hostname}:18234/connect/${this.id}?token=${this.authService.token}`
    );
    this.websocket.addEventListener('message', (event) => {
      let newEvent: RequestData = JSON.parse(event.data);
      newEvent.time = new Date(newEvent.time);
      if (!this.selectedEvent) this.selectedEvent = newEvent;
      this.events = [newEvent, ...this.events];
    });
    this.websocket.addEventListener('close', (event) => {
      switch (event.code) {
        case 1001: // Going Away
          this._snackbar.open(`Server is shutting down.`, 'Ok', {
            duration: 5000,
          });
          this.authService.clearToken();
          this.broken = true;
          break;
        case 4001: // Unauthorized
          this._snackbar.open(`Session has expired.`, 'Ok', {
            duration: 5000,
          });
          this.authService.clearToken();
          this.broken = true;
          break;
        default: // Anythign else
          this._snackbar.open(`WebSocket has been closed.`, 'Ok', {
            duration: 5000,
          });
      }
    });
  }
}
