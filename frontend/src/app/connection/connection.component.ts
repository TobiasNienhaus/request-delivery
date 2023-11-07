import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { validate as validateUuid } from 'uuid';
import { RequestData, DEFAULT_REQUEST } from '../model/request';
import { MatSnackBar } from '@angular/material/snack-bar';
import { IClipboardResponse } from 'ngx-clipboard';

@Component({
  selector: 'app-connection',
  templateUrl: './connection.component.html',
  styleUrls: ['./connection.component.scss'],
})
export class ConnectionComponent {
  selectedEvent?: RequestData;
  websocket?: WebSocket;
  id?: string;

  events: RequestData[] = [];

  constructor(private route: ActivatedRoute, private _snackbar: MatSnackBar) {
    this.selectedEvent = this.events[0];
    this.route.paramMap.subscribe((params) => {
      let id = params.get('id');
      if (validateUuid(id || '')) {
        this.id = id || undefined;
        this.websocket = new WebSocket(`ws://localhost:18234/connect/${id}`);
        this.websocket.addEventListener('message', (event) => {
          let newEvent: RequestData = JSON.parse(event.data);
          newEvent.time = new Date(newEvent.time);
          if (!this.selectedEvent) this.selectedEvent = newEvent;
          this.events = [newEvent, ...this.events];
        });
      } else {
        console.error('Wrong ID Format');
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
}
