import { Component } from '@angular/core';
import { PluginDashboardComponent } from '../shared/plugin-dashboard/plugin-dashboard.component';
import { PluginFeatureDescription } from '../shared/plugin-feature.model';
import { PLUGIN_FEATURES } from '../shared/plugin-features';

@Component({
  selector: 'app-candle-dashboard',
  standalone: true,
  imports: [PluginDashboardComponent],
  template: `
    <app-plugin-dashboard
      [pluginId]="pluginId"
      [featureCards]="features"
    ></app-plugin-dashboard>
  `,
})
export class CandleDashboardComponent {
  protected readonly pluginId = 'candle';
  protected readonly features: PluginFeatureDescription[] = PLUGIN_FEATURES['candle'];
}
