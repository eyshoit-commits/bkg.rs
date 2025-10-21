import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ApiService } from '../../services/api.service';
import { ChatMessage } from '../../models/api.models';
import { firstValueFrom } from 'rxjs';

@Component({
  selector: 'app-chat',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './chat.component.html',
  styleUrls: ['./chat.component.css']
})
export class ChatComponent {
  messages: ChatMessage[] = [{ role: 'system', content: 'You are interacting with the BKG orchestration stack.' }];
  userInput = '';
  loading = false;
  error: string | null = null;

  constructor(private readonly api: ApiService) {}

  async sendMessage(): Promise<void> {
    const content = this.userInput.trim();
    if (!content || this.loading) {
      return;
    }
    this.error = null;
    this.userInput = '';
    const userMessage: ChatMessage = { role: 'user', content };
    this.messages = [...this.messages, userMessage];
    this.loading = true;
    try {
      const response = await firstValueFrom(this.api.chat(this.messages));
      const assistant = response?.choices?.[0]?.message;
      if (assistant) {
        this.messages = [...this.messages, assistant];
      }
    } catch (error) {
      this.error = (error as Error).message;
    } finally {
      this.loading = false;
    }
  }
}
