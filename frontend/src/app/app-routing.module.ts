import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { ConnectionComponent } from './connection/connection.component';
import { MainAppComponent } from './main-app/main-app.component';

const routes: Routes = [
  {
    path: '',
    component: MainAppComponent,
  },
  {
    path: 'results/:id',
    component: ConnectionComponent,
  },
  {
    path: '**',
    redirectTo: '/',
  },
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule],
})
export class AppRoutingModule {}
