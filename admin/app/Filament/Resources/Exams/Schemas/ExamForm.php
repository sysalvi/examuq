<?php

namespace App\Filament\Resources\Exams\Schemas;

use Filament\Forms\Components\DateTimePicker;
use Filament\Forms\Components\Textarea;
use Filament\Forms\Components\TextInput;
use Filament\Forms\Components\Toggle;
use Filament\Schemas\Schema;

class ExamForm
{
    public static function configure(Schema $schema): Schema
    {
        return $schema
            ->components([
                TextInput::make('title')
                    ->required()
                    ->maxLength(255),
                TextInput::make('subject')
                    ->maxLength(255),
                TextInput::make('class_room')
                    ->label('Kelas / Ruang')
                    ->required()
                    ->maxLength(255),
                Textarea::make('exam_url')
                    ->label('URL Ujian')
                    ->required()
                    ->rows(2)
                    ->columnSpanFull(),
                DateTimePicker::make('start_at')
                    ->label('Mulai Ujian')
                    ->seconds(false),
                DateTimePicker::make('end_at')
                    ->label('Selesai Ujian')
                    ->seconds(false),
                TextInput::make('duration_minutes')
                    ->label('Durasi (menit)')
                    ->numeric()
                    ->minValue(1)
                    ->maxValue(600),
                TextInput::make('token_global')
                    ->required()
                    ->maxLength(255),
                Toggle::make('is_active')
                    ->label('Aktif')
                    ->default(true),
            ]);
    }
}
