import { Component } from '@angular/core';
import { Router, RouterLink, RouterOutlet } from '@angular/router';
import { NgClass, NgFor, NgIf } from '@angular/common';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, RouterLink, NgClass, NgFor, NgIf],
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})
export class AppComponent {
  title = 'BKG Control Center';
  tabs = [
    { path: '/chat', label: 'Chat' },
    { path: '/plugins', label: 'Plugins' },
    { path: '/admin', label: 'Admin' }
  ];

  constructor(private readonly router: Router) {}

  isActive(path: string): boolean {
    return this.router.url.startsWith(path);
  }
}
