<?php

namespace App\Filament\Resources\Exams\Tables;

use App\Filament\Resources\ExamSessions\ExamSessionResource;
use App\Models\Exam;
use Filament\Actions\Action;
use Filament\Actions\BulkActionGroup;
use Filament\Actions\DeleteBulkAction;
use Filament\Actions\EditAction;
use Filament\Tables\Columns\TextColumn;
use Filament\Tables\Table;

class ExamsTable
{
    public static function configure(Table $table): Table
    {
        return $table
            ->columns([
                TextColumn::make('title')
                    ->label('Judul')
                    ->searchable()
                    ->sortable(),
                TextColumn::make('class_room')
                    ->label('Kelas/Ruang')
                    ->searchable(),
                TextColumn::make('token_global')
                    ->label('Token')
                    ->searchable()
                    ->badge()
                    ->color('warning')
                    ->copyable()
                    ->copyMessage('Token ujian berhasil disalin')
                    ->copyMessageDuration(1500)
                    ->fontFamily('mono')
                    ->weight('bold')
                    ->alignCenter(),
                TextColumn::make('exam_url')
                    ->label('URL Ujian')
                    ->wrap()
                    ->limit(50),
                TextColumn::make('is_active')
                    ->label('Status')
                    ->badge()
                    ->formatStateUsing(fn (bool $state): string => $state ? 'Aktif' : 'Nonaktif')
                    ->color(fn (bool $state): string => $state ? 'success' : 'gray'),
                TextColumn::make('start_at')
                    ->label('Mulai')
                    ->dateTime()
                    ->sortable(),
                TextColumn::make('end_at')
                    ->label('Selesai')
                    ->dateTime()
                    ->sortable(),
            ])
            ->filters([])
            ->recordActions([
                Action::make('sessions')
                    ->label('Sesi')
                    ->url(fn (Exam $record): string => ExamSessionResource::getUrl('index', ['exam_id' => $record->id])),
                EditAction::make(),
            ])
            ->toolbarActions([
                BulkActionGroup::make([
                    DeleteBulkAction::make(),
                ]),
            ]);
    }
}
