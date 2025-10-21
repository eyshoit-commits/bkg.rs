import { Component } from '@angular/core';
import { PluginDashboardComponent } from '../shared/plugin-dashboard/plugin-dashboard.component';
import { PluginFeatureDescription } from '../shared/plugin-feature.model';
import { PLUGIN_FEATURES } from '../shared/plugin-features';

@Component({
  selector: 'app-brainml-dashboard',
  standalone: true,
  imports: [PluginDashboardComponent],
  template: `
    <app-plugin-dashboard
      [pluginId]="pluginId"
      [featureCards]="features"
    ></app-plugin-dashboard>
  `,
})
export class BrainmlDashboardComponent {
  protected readonly pluginId = 'brainml';
  protected readonly features: PluginFeatureDescription[] = PLUGIN_FEATURES['brainml'];
}
