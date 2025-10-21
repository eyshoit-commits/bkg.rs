import { IsArray, IsOptional, IsString, ValidateNested } from 'class-validator';
import { Type } from 'class-transformer';

export class ChatMessageDto {
  @IsString()
  role!: 'system' | 'user' | 'assistant';

  @IsString()
  content!: string;
}

export class ChatCompletionDto {
  @IsArray()
  @ValidateNested({ each: true })
  @Type(() => ChatMessageDto)
  messages!: ChatMessageDto[];

  @IsOptional()
  @IsString()
  model?: string;
}

export class EmbeddingRequestDto {
  @IsArray()
  @IsString({ each: true })
  input!: string[];

  @IsOptional()
  @IsString()
  model?: string;
}
