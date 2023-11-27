import { Component, Input, OnInit, SimpleChanges } from '@angular/core';
import { RequestData } from '../model/request';

import { BodyType, getBodyType } from '../model/format';

import { XMLParser } from 'fast-xml-parser/src/fxp';

@Component({
  selector: 'app-request-body',
  templateUrl: './request-body.component.html',
  styleUrls: ['./request-body.component.scss'],
})
export class RequestBodyComponent {
  private _requestEvent!: RequestData;
  @Input() set requestEvent(value: RequestData) {
    this._requestEvent = value;
    this.formattedBody = this.getFormattedBody();
  }

  get requestEvent(): RequestData {
    return this._requestEvent;
  }

  formattedBody: any = null;

  get bodyType(): BodyType {
    return getBodyType(this.requestEvent.contentType || '');
  }

  constructor() {}

  getFormattedBody(): any {
    if (!this.requestEvent.body?.raw) return {};
    switch (this.bodyType) {
      case BodyType.Json:
        return JSON.parse(this.requestEvent.body.raw);
      case BodyType.Xml:
        return new XMLParser().parse(this.requestEvent.body.raw);
      default:
        return {};
    }
  }

  hasBody(): boolean {
    return (
      this.requestEvent.body !== null &&
      this.bodyType in [BodyType.Json, BodyType.Xml]
    );
  }
}
