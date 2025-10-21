import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { ChatComponent } from './components/chat/chat.component';
import { PluginsComponent } from './components/plugins/plugins.component';
import { AdminComponent } from './components/admin/admin.component';

const routes: Routes = [
  { path: 'chat', component: ChatComponent },
  { path: 'plugins', component: PluginsComponent },
  { path: 'admin', component: AdminComponent },
  { path: '', redirectTo: 'chat', pathMatch: 'full' }
];

@NgModule({
  imports: [RouterModule.forRoot(routes, { bindToComponentInputs: true })],
  exports: [RouterModule]
})
export class AppRoutingModule {}
