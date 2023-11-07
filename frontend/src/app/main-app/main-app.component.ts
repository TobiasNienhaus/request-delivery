import { Component } from '@angular/core';
import { v4 } from 'uuid';

@Component({
  selector: 'app-main-app',
  templateUrl: './main-app.component.html',
  styleUrls: ['./main-app.component.scss'],
})
export class MainAppComponent {
  get uuid(): string {
    return v4();
  }
}
