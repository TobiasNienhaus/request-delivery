import { Component, Input } from '@angular/core';
import { RequestData } from '../model/request';
import { canShowInteractive } from '../model/format';

@Component({
  selector: 'app-request',
  templateUrl: './request.component.html',
  styleUrls: ['./request.component.scss'],
})
export class RequestComponent {
  @Input() requestEvent!: RequestData;

  get headers(): { header: string; values: string[] }[] {
    let ret: { header: string; values: string[] }[] = [];

    for (const [key, value] of Object.entries(this.requestEvent.headers)) {
      ret.push({ header: key, values: value });
    }
    return ret;
  }

  get cookies(): { cookie: string; values: string[] }[] {
    let ret: { cookie: string; values: string[] }[] = [];

    for (const [key, value] of Object.entries(this.requestEvent.cookies)) {
      ret.push({ cookie: key, values: value });
    }
    return ret;
  }

  get canShowFormatted(): boolean {
    return this.requestEvent?.contentType
      ? canShowInteractive(this.requestEvent.contentType)
      : false;
  }
}
