import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { ConnectionComponent } from './connection/connection.component';
import { MainAppComponent } from './main-app/main-app.component';
import { hasValidToken, reconnect } from './app.guards';

const routes: Routes = [
  {
    path: '',
    component: MainAppComponent,
  },
  {
    path: 'results/:id',
    component: ConnectionComponent,
    canActivate: [hasValidToken],
  },
  {
    path: 'reconnect/:id',
    component: ConnectionComponent,
    canActivate: [reconnect],
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
