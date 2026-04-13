<?php

namespace App\Filament\Resources\ExamSessions\Tables;

use App\Models\ExamSession;
use Carbon\Carbon;
use Filament\Actions\Action;
use Filament\Tables\Columns\TextColumn;
use Filament\Tables\Filters\SelectFilter;
use Filament\Tables\Table;
use Illuminate\Database\Eloquent\Builder;

class ExamSessionsTable
{
    public static function configure(Table $table): Table
    {
        return $table
            ->poll('45s')
            ->modifyQueryUsing(function (Builder $query): Builder {
                $examId = request()->integer('exam_id');

                if ($examId > 0) {
                    $query->where('exam_id', $examId);
                }

                return $query;
            })
            ->columns([
                TextColumn::make('id')
                    ->label('ID')
                    ->sortable(),
                TextColumn::make('exam.title')
                    ->label('Ujian')
                    ->searchable()
                    ->sortable(),
                TextColumn::make('display_name')
                    ->label('Nama')
                    ->searchable(),
                TextColumn::make('class_room')
                    ->label('Kelas/Ruang')
                    ->searchable(),
                TextColumn::make('client_type')
                    ->label('Client')
                    ->badge(),
                TextColumn::make('status')
                    ->badge()
                    ->color(function (string $state): string {
                        return match ($state) {
                            'active' => 'success',
                            'finished' => 'gray',
                            'blocked' => 'danger',
                            default => 'warning',
                        };
                    }),
                TextColumn::make('last_heartbeat_at')
                    ->label('Heartbeat')
                    ->since(),
                TextColumn::make('started_at')
                    ->label('Mulai')
                    ->dateTime()
                    ->sortable(),
                TextColumn::make('ended_at')
                    ->label('Selesai')
                    ->dateTime()
                    ->placeholder('-'),
            ])
            ->filters([
                SelectFilter::make('status')
                    ->options([
                        'active' => 'Active',
                        'finished' => 'Finished',
                        'blocked' => 'Blocked',
                        'disconnected' => 'Disconnected',
                    ]),
                SelectFilter::make('client_type')
                    ->label('Client')
                    ->options([
                        'desktop_client' => 'Desktop Client',
                        'chrome_extension' => 'Chrome Extension',
                    ]),
                SelectFilter::make('exam_id')
                    ->label('Ujian')
                    ->relationship('exam', 'title')
                    ->searchable()
                    ->preload(),
            ])
            ->recordActions([
                Action::make('forceEnd')
                    ->label('Force End')
                    ->color('danger')
                    ->requiresConfirmation()
                    ->visible(fn (ExamSession $record): bool => $record->status !== 'finished')
                    ->action(function (ExamSession $record): void {
                        $now = Carbon::now();

                        $record->update([
                            'status' => 'finished',
                            'ended_at' => $now,
                            'last_heartbeat_at' => $now,
                        ]);
                    }),
            ])
            ->defaultSort('started_at', 'desc');
    }
}
