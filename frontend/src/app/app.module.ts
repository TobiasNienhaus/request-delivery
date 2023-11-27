import { CUSTOM_ELEMENTS_SCHEMA, NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';
import { MatButtonModule } from '@angular/material/button';
import { MatSidenavModule } from '@angular/material/sidenav';
import { MatExpansionModule } from '@angular/material/expansion';
import { MatTableModule } from '@angular/material/table';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatListModule } from '@angular/material/list';
import { MatIconModule } from '@angular/material/icon';
import { MatSnackBarModule } from '@angular/material/snack-bar';

import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { ConnectionComponent } from './connection/connection.component';
import { NgxJsonViewerModule } from 'ngx-json-viewer';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';
import { MainAppComponent } from './main-app/main-app.component';
import { RequestComponent } from './request/request.component';
import { RequestBodyComponent } from './request-body/request-body.component';
import { UrlFormTableComponent } from './url-form-table/url-form-table.component';

import { ClipboardModule } from 'ngx-clipboard';

import {
  HIGHLIGHT_OPTIONS,
  HighlightModule,
  HighlightOptions,
} from 'ngx-highlightjs';
import { HttpClientModule } from '@angular/common/http';
import { LayoutModule } from '@angular/cdk/layout';
import { CookieService } from 'ngx-cookie-service';
import { MatBottomSheetModule } from '@angular/material/bottom-sheet';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { ConnectionInfoBottomSheetComponent } from './connection/connection-info-bottom-sheet/connection-info-bottom-sheet.component';
import { QrCodeModule } from 'ng-qrcode';
import { MatSelectModule } from '@angular/material/select';
import { ReactiveFormsModule } from '@angular/forms';

@NgModule({
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  declarations: [
    AppComponent,
    ConnectionComponent,
    MainAppComponent,
    RequestComponent,
    RequestBodyComponent,
    UrlFormTableComponent,
    ConnectionInfoBottomSheetComponent,
  ],
  imports: [
    MatButtonModule,
    BrowserModule,
    AppRoutingModule,
    NgxJsonViewerModule,
    BrowserAnimationsModule,
    MatSidenavModule,
    MatExpansionModule,
    MatTableModule,
    MatToolbarModule,
    MatListModule,
    MatIconModule,
    HighlightModule,
    ClipboardModule,
    MatSnackBarModule,
    HttpClientModule,
    LayoutModule,
    MatBottomSheetModule,
    MatFormFieldModule,
    MatInputModule,
    QrCodeModule,
    MatSelectModule,
    ReactiveFormsModule,
  ],
  providers: [
    {
      provide: HIGHLIGHT_OPTIONS,
      useValue: <HighlightOptions>{
        fullLibraryLoader: () => import('highlight.js'),
        lineNumbersLoader: () => import('ngx-highlightjs/line-numbers'),
      },
    },
    CookieService,
  ],
  bootstrap: [AppComponent],
})
export class AppModule {}

// TODO QR Code with credentials so you can view the calls on another device
// TODO Option to copy X-Auth-Token
