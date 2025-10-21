import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { ChatComponent } from './components/chat/chat.component';
import { AdminComponent } from './components/admin/admin.component';
import { PluginListComponent } from './features/plugins/plugin-list/plugin-list.component';
import { PluginDashboardComponent } from './features/plugins/plugin-dashboard/plugin-dashboard.component';

const routes: Routes = [
  { path: 'chat', component: ChatComponent },
  {
    path: 'plugins',
    children: [
      { path: '', component: PluginListComponent },
      { path: ':pluginId', component: PluginDashboardComponent }
    ]
  },
  { path: 'admin', component: AdminComponent },
  { path: '', redirectTo: 'chat', pathMatch: 'full' }
];

@NgModule({
  imports: [RouterModule.forRoot(routes, { bindToComponentInputs: true })],
  exports: [RouterModule]
})
export class AppRoutingModule {}
