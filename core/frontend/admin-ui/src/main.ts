import { enableProdMode, importProvidersFrom } from '@angular/core';
import { bootstrapApplication } from '@angular/platform-browser';
import { HTTP_INTERCEPTORS, provideHttpClient, withInterceptorsFromDi } from '@angular/common/http';
import { AppComponent } from './app/app.component';
import { AppRoutingModule } from './app/app-routing.module';
import { AuthInterceptor } from './app/services/auth.interceptor';

if (process.env['NODE_ENV'] === 'production') {
  enableProdMode();
}

bootstrapApplication(AppComponent, {
  providers: [
    importProvidersFrom(AppRoutingModule),
    provideHttpClient(withInterceptorsFromDi()),
    { provide: HTTP_INTERCEPTORS, useClass: AuthInterceptor, multi: true },
  ],
}).catch((err) => console.error(err));
