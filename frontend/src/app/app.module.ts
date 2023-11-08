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
import { APP_BASE_HREF } from '@angular/common';

@NgModule({
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  declarations: [
    AppComponent,
    ConnectionComponent,
    MainAppComponent,
    RequestComponent,
    RequestBodyComponent,
    UrlFormTableComponent,
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
  ],
  providers: [
    {
      provide: HIGHLIGHT_OPTIONS,
      useValue: <HighlightOptions>{
        fullLibraryLoader: () => import('highlight.js'),
        lineNumbersLoader: () => import('ngx-highlightjs/line-numbers'), // Optional, only if you want the line numbers
        // languages: {
        //   javascript: () => import('highlight.js/lib/languages/javascript'),
        //   css: () => import('highlight.js/lib/languages/css'),
        //   xml: () => import('highlight.js/lib/languages/xml'),
        //   plaintext: () => import('highlight.js/lib/languages/plaintext'),
        //   graphql: () => import('highlight.js/lib/languages/graphql'),
        // },
      },
    },
  ],
  bootstrap: [AppComponent],
})
export class AppModule {}
