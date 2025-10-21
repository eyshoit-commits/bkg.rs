import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { ChatComponent } from './components/chat/chat.component';
import { AdminComponent } from './components/admin/admin.component';
import { PluginListComponent } from './features/plugins/shared/plugin-list/plugin-list.component';
import { PluginDashboardComponent } from './features/plugins/shared/plugin-dashboard/plugin-dashboard.component';
import { BrainmlDashboardComponent } from './features/plugins/brainml/brainml-dashboard.component';
import { CandleDashboardComponent } from './features/plugins/candle/candle-dashboard.component';
import { RustyfaceDashboardComponent } from './features/plugins/rustyface/rustyface-dashboard.component';
import { LlmserverDashboardComponent } from './features/plugins/llmserver/llmserver-dashboard.component';
import { RepoagentDashboardComponent } from './features/plugins/repoagent/repoagent-dashboard.component';
import { ApikeysDashboardComponent } from './features/plugins/apikeys/apikeys-dashboard.component';

const routes: Routes = [
  { path: 'chat', component: ChatComponent },
  {
    path: 'plugins',
    children: [
      { path: '', component: PluginListComponent },
      { path: 'brainml', component: BrainmlDashboardComponent },
      { path: 'candle', component: CandleDashboardComponent },
      { path: 'rustyface', component: RustyfaceDashboardComponent },
      { path: 'llmserver', component: LlmserverDashboardComponent },
      { path: 'repoagent', component: RepoagentDashboardComponent },
      { path: 'apikeys', component: ApikeysDashboardComponent },
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
