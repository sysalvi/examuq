<?php

namespace App\Filament\Resources\Exams\Schemas;

use Filament\Forms\Components\DateTimePicker;
use Filament\Forms\Components\Placeholder;
use Filament\Forms\Components\Textarea;
use Filament\Forms\Components\TextInput;
use Filament\Forms\Components\Toggle;
use Filament\Schemas\Components\Section;
use Illuminate\Support\HtmlString;
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
                Section::make('Token Ujian')
                    ->description('Token dibuat otomatis saat ujian disimpan dan dipakai siswa untuk masuk ujian.')
                    ->schema([
                        Placeholder::make('token_global_info')
                            ->hiddenLabel()
                            ->content(fn ($record): HtmlString => new HtmlString($record?->token_global
                                ? '<div style="text-align:center;padding:1rem;border-radius:0.75rem;background:#fff7ed;border:2px solid #fdba74;">'
                                    . '<div style="font-size:0.875rem;font-weight:600;color:#9a3412;margin-bottom:0.5rem;">TOKEN UJIAN</div>'
                                    . '<div style="font-size:2rem;line-height:1;font-weight:800;letter-spacing:0.35em;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;color:#7c2d12;">'
                                    . e($record->token_global)
                                    . '</div>'
                                    . '<div style="margin-top:0.75rem;font-size:0.875rem;color:#9a3412;">Salin token ini dan bagikan ke siswa.</div>'
                                    . '</div>'
                                : '<div style="text-align:center;padding:1rem;border-radius:0.75rem;background:#f8fafc;border:1px dashed #cbd5e1;color:#475569;">Akan dibuat otomatis saat ujian disimpan.</div>'))
                            ->columnSpanFull(),
                    ])
                    ->columnSpanFull(),
                Toggle::make('is_active')
                    ->label('Aktif')
                    ->default(true),
            ]);
    }
}
