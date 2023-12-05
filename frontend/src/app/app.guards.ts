import {
  ActivatedRouteSnapshot,
  CanActivateFn,
  Router,
  RouterStateSnapshot,
} from '@angular/router';
import { AuthService } from './service/auth.service';
import { inject } from '@angular/core';

export const hasValidToken: CanActivateFn = async (
  route: ActivatedRouteSnapshot,
  _: RouterStateSnapshot
) => {
  let id = route.paramMap.get('id');
  let router = inject(Router);
  if (id !== null && (await inject(AuthService).validateStored(id))) {
    return true;
  } else {
    return router.createUrlTree(['/'], { relativeTo: null });
  }
};

export const reconnect: CanActivateFn = async (
  route: ActivatedRouteSnapshot,
  _: RouterStateSnapshot
) => {
  let id = route.paramMap.get('id');
  let token = route.queryParams['token'];
  let authService = inject(AuthService);
  let router = inject(Router);
  if (id !== null && (await authService.validate(id, token))) {
    authService.setToken(token || '');
    return router.createUrlTree(['/results', id], {
      relativeTo: null,
    });
  } else {
    return router.createUrlTree(['/'], { relativeTo: null });
  }
};
