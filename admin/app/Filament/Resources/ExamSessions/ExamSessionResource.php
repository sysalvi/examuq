<?php

namespace App\Filament\Resources\ExamSessions;

use App\Filament\Resources\ExamSessions\Pages\ListExamSessions;
use App\Filament\Resources\ExamSessions\Schemas\ExamSessionForm;
use App\Filament\Resources\ExamSessions\Tables\ExamSessionsTable;
use App\Models\ExamSession;
use BackedEnum;
use UnitEnum;
use Filament\Resources\Resource;
use Filament\Schemas\Schema;
use Filament\Support\Icons\Heroicon;
use Filament\Tables\Table;

class ExamSessionResource extends Resource
{
    protected static ?string $model = ExamSession::class;

    protected static string|BackedEnum|null $navigationIcon = Heroicon::OutlinedComputerDesktop;

    protected static ?string $navigationLabel = 'Sesi Ujian';

    protected static string|UnitEnum|null $navigationGroup = 'Monitoring';

    protected static ?int $navigationSort = 1;

    public static function form(Schema $schema): Schema
    {
        return ExamSessionForm::configure($schema);
    }

    public static function table(Table $table): Table
    {
        return ExamSessionsTable::configure($table);
    }

    public static function getRelations(): array
    {
        return [];
    }

    public static function getPages(): array
    {
        return [
            'index' => ListExamSessions::route('/'),
        ];
    }

    public static function canCreate(): bool
    {
        return false;
    }
}
