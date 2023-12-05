import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatBottomSheet } from '@angular/material/bottom-sheet';
import { LegalBottomSheetComponent } from '../legal-bottom-sheet/legal-bottom-sheet.component';

@Component({
  selector: 'app-legal-link',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './legal-link.component.html',
  styleUrl: './legal-link.component.scss',
})
export class LegalLinkComponent {
  constructor(private _bottomSheet: MatBottomSheet) {}

  openBottonSheet() {
    this._bottomSheet.open(LegalBottomSheetComponent);
  }
}
