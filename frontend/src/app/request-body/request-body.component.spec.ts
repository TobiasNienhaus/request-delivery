import { ComponentFixture, TestBed } from '@angular/core/testing';

import { RequestBodyComponent } from './request-body.component';

describe('RequestBodyComponent', () => {
  let component: RequestBodyComponent;
  let fixture: ComponentFixture<RequestBodyComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      declarations: [RequestBodyComponent]
    });
    fixture = TestBed.createComponent(RequestBodyComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
