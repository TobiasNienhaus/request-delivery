import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
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

@Component({
  selector: 'app-connection',
  templateUrl: './connection.component.html',
  styleUrls: ['./connection.component.scss'],
})
export class ConnectionComponent {
  selectedEvent?: RequestData;
  websocket?: WebSocket;
  id?: string;
  broken = true;

  events: RequestData[] = [
    { ...DEFAULT_REQUEST },
    { ...DEFAULT_REQUEST, method: 'DELETE' },
    { ...DEFAULT_REQUEST, method: 'HEAD' },
    { ...DEFAULT_REQUEST, method: 'PATCH' },
    { ...DEFAULT_REQUEST, method: 'PUT' },
    { ...DEFAULT_REQUEST },
    { ...DEFAULT_REQUEST, method: 'DELETE' },
    { ...DEFAULT_REQUEST, method: 'HEAD' },
    { ...DEFAULT_REQUEST, method: 'PATCH' },
    { ...DEFAULT_REQUEST, method: 'PUT' },
    { ...DEFAULT_REQUEST },
    { ...DEFAULT_REQUEST, method: 'DELETE' },
    { ...DEFAULT_REQUEST, method: 'HEAD' },
    { ...DEFAULT_REQUEST, method: 'PATCH' },
    { ...DEFAULT_REQUEST, method: 'PUT' },
    { ...DEFAULT_REQUEST },
    { ...DEFAULT_REQUEST, method: 'DELETE' },
    { ...DEFAULT_REQUEST, method: 'HEAD' },
    { ...DEFAULT_REQUEST, method: 'PATCH' },
    { ...DEFAULT_REQUEST, method: 'PUT' },
    { ...DEFAULT_REQUEST },
    { ...DEFAULT_REQUEST, method: 'DELETE' },
    { ...DEFAULT_REQUEST, method: 'HEAD' },
    { ...DEFAULT_REQUEST, method: 'PATCH' },
    { ...DEFAULT_REQUEST, method: 'PUT' },
    { ...DEFAULT_REQUEST },
    { ...DEFAULT_REQUEST, method: 'DELETE' },
    { ...DEFAULT_REQUEST, method: 'HEAD' },
    { ...DEFAULT_REQUEST, method: 'PATCH' },
    { ...DEFAULT_REQUEST, method: 'PUT' },
    { ...DEFAULT_REQUEST },
    { ...DEFAULT_REQUEST, method: 'DELETE' },
    { ...DEFAULT_REQUEST, method: 'HEAD' },
    { ...DEFAULT_REQUEST, method: 'PATCH' },
    { ...DEFAULT_REQUEST, method: 'PUT' },
    { ...DEFAULT_REQUEST },
    { ...DEFAULT_REQUEST, method: 'DELETE' },
    { ...DEFAULT_REQUEST, method: 'HEAD' },
    { ...DEFAULT_REQUEST, method: 'PATCH' },
    { ...DEFAULT_REQUEST, method: 'PUT' },
  ];

  constructor(
    private route: ActivatedRoute,
    private _snackbar: MatSnackBar,
    private _bottomSheet: MatBottomSheet,
    private authService: AuthService
  ) {
    if (!authService.has_token()) {
      return;
    }
    this.selectedEvent = this.events[0];
    this.route.paramMap.subscribe((params) => {
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
      this.websocket.close();
      this._snackbar.open('Closed WebSocket Connection', 'Ok', {
        duration: 5000,
      });
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

  createNewWebsocketConnection() {
    this.websocket = new WebSocket(
      `ws://localhost:18234/connect/${this.id}?token=${this.authService.token}`
    );
    this.websocket.addEventListener('message', (event) => {
      let newEvent: RequestData = JSON.parse(event.data);
      newEvent.time = new Date(newEvent.time);
      if (!this.selectedEvent) this.selectedEvent = newEvent;
      this.events = [newEvent, ...this.events];
    });
  }
}
