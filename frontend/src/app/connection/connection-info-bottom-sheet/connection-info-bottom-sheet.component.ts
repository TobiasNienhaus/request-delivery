import { Component, Inject } from '@angular/core';
import { MAT_BOTTOM_SHEET_DATA } from '@angular/material/bottom-sheet';
import { AuthService } from 'src/app/service/auth.service';
import { IClipboardResponse } from 'ngx-clipboard';
import { MatSnackBar } from '@angular/material/snack-bar';
import { Location } from '@angular/common';

@Component({
  selector: 'app-connection-info-bottom-sheet',
  templateUrl: './connection-info-bottom-sheet.component.html',
  styleUrl: './connection-info-bottom-sheet.component.scss',
})
export class ConnectionInfoBottomSheetComponent {
  constructor(
    @Inject(MAT_BOTTOM_SHEET_DATA) public data: { id: string },
    private authService: AuthService,
    private _snackbar: MatSnackBar,
    private location: Location
  ) {}

  get token(): string | undefined {
    return this.authService.token;
  }

  get id(): string {
    return this.data.id;
  }
  get qrUrl(): string {
    return (
      window.location.origin +
      this.location.prepareExternalUrl(
        `/reconnect/${this.id}` + (this.token ? `?token=${this.token}` : '')
      )
    );
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
