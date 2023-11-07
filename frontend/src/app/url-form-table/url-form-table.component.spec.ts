import { ComponentFixture, TestBed } from '@angular/core/testing';

import { UrlFormTableComponent } from './url-form-table.component';

describe('UrlFormTableComponent', () => {
  let component: UrlFormTableComponent;
  let fixture: ComponentFixture<UrlFormTableComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      declarations: [UrlFormTableComponent]
    });
    fixture = TestBed.createComponent(UrlFormTableComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
